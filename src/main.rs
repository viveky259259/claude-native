use std::process;

use anyhow::Result;
use clap::Parser;
use owo_colors::OwoColorize;

use claude_native::cli::{Cli, OutputFormat};
use claude_native::{config, detection, diff, fix, history, hooks, init, mcp, multi_tool, output, rules, scan, scoring, token_cost, watch};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {e:#}");
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let path = cli.path.canonicalize().unwrap_or_else(|_| cli.path.clone());
    if !path.is_dir() {
        anyhow::bail!("{} is not a directory", path.display());
    }

    // Modes that don't need full scanning
    if let Some(min_score) = cli.hook { return hooks::install_hook(&path, min_score); }
    if cli.mcp { return mcp::serve(); }
    if cli.watch { return watch::watch_and_score(&path); }
    if cli.diff { return diff::show_diff(&path); }

    // Scan and detect
    let mut ctx = scan::build_context(&path)?;
    let pt = detection::detect(&ctx);
    ctx.project_type = Some(pt.clone());

    // Init modes
    if cli.init_all { return run_init_all(&ctx); }
    if cli.init { return run_init(&ctx, &pt); }

    // Score and output
    run_score_and_output(&cli, &ctx, &pt, &path)
}

fn run_init_all(ctx: &scan::ProjectContext) -> Result<()> {
    let created = multi_tool::generate_all(ctx)?;
    if created.is_empty() {
        println!("{}", "All config files already exist.".dimmed());
    } else {
        println!("{}", "Created configs for Claude, Cursor, and Copilot:".green().bold());
        for f in &created { println!("  {} {f}", "+".green()); }
    }
    Ok(())
}

fn run_init(ctx: &scan::ProjectContext, pt: &detection::ProjectType) -> Result<()> {
    let created = init::init_project(ctx)?;
    if created.is_empty() {
        println!("{}", "All Claude Native files already exist.".dimmed());
    } else {
        println!("{}", "Created Claude Native files:".green().bold());
        for f in &created { println!("  {} {f}", "+".green()); }
        println!();
        println!("Detected: {}", pt.to_string().bold());
        println!("Run {} to see your score.", "claude-native".bold());
    }
    Ok(())
}

fn run_score_and_output(
    cli: &Cli, ctx: &scan::ProjectContext,
    pt: &detection::ProjectType, path: &std::path::Path,
) -> Result<()> {
    let cfg = config::Config::load(path);
    let disabled = cfg.disabled_set();

    let all = rules::all_rules();
    let results: Vec<_> = all.iter()
        .filter(|r| r.applies_to(pt) && !disabled.contains(r.id()))
        .map(|r| r.check(ctx))
        .collect();
    let mut sc = scoring::calculate(results, pt);
    token_cost::enrich_suggestions(&mut sc.suggestions, &sc.rule_results, ctx);

    if cli.fix { return run_fix(ctx, &sc, path); }
    if cli.badge { print_badge(&sc); return Ok(()); }
    if cli.history {
        render_output(&sc, &cli.format)?;
        history::record_and_show(&sc, path)?;
        return Ok(());
    }

    render_output(&sc, &cli.format)?;
    if sc.total_score < 40.0 { process::exit(1); }
    Ok(())
}

fn run_fix(
    ctx: &scan::ProjectContext, sc: &scoring::Scorecard, path: &std::path::Path,
) -> Result<()> {
    let actions = fix::apply_fixes(ctx, &sc.rule_results)?;
    if actions.is_empty() {
        println!("{}", "Nothing to fix — all quick wins already applied.".dimmed());
        return Ok(());
    }
    println!("{}", "Applied fixes:".green().bold());
    for a in &actions { println!("  {} {a}", "+".green()); }

    println!();
    let mut new_ctx = scan::build_context(path)?;
    let new_pt = detection::detect(&new_ctx);
    new_ctx.project_type = Some(new_pt.clone());
    let new_results: Vec<_> = rules::all_rules().iter()
        .filter(|r| r.applies_to(&new_pt))
        .map(|r| r.check(&new_ctx))
        .collect();
    let new_sc = scoring::calculate(new_results, &new_pt);
    let delta = new_sc.total_score - sc.total_score;
    println!("Score: {:.0} -> {}  ({})", sc.total_score,
        format!("{:.0}", new_sc.total_score).bold(), format!("+{:.0}", delta).green());
    Ok(())
}

fn print_badge(sc: &scoring::Scorecard) {
    let score = sc.total_score as u32;
    let grade = format!("{}", sc.grade);
    let color = match sc.grade {
        scoring::Grade::APlus | scoring::Grade::A => "brightgreen",
        scoring::Grade::B => "green",
        scoring::Grade::C => "yellow",
        scoring::Grade::D => "orange",
        scoring::Grade::F => "red",
    };
    let url = format!("https://img.shields.io/badge/claude--native-{grade}%20({score}%2F100)-{color}");
    println!("Badge URL:\n  {url}\n\nMarkdown:\n  ![Claude Native Score]({url})\n\nHTML:\n  <img src=\"{url}\" alt=\"Claude Native Score\">");
}

fn render_output(sc: &scoring::Scorecard, fmt: &OutputFormat) -> Result<()> {
    match fmt {
        OutputFormat::Terminal => output::print_scorecard(sc),
        OutputFormat::Json => {
            let json = serde_json::json!({
                "project_type": format!("{}", sc.project_type),
                "score": sc.total_score,
                "grade": format!("{}", sc.grade),
                "dimensions": sc.dimensions.iter().map(|d| serde_json::json!({
                    "name": format!("{}", d.dimension),
                    "score": d.score, "weight": d.weight,
                    "passed": d.rules_passed, "failed": d.rules_failed,
                    "warned": d.rules_warned,
                })).collect::<Vec<_>>(),
                "suggestions_count": sc.suggestions.len(),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
    }
    Ok(())
}
