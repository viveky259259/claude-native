use anyhow::Result;
use owo_colors::OwoColorize;

use crate::detection;
use crate::fix;
use crate::rules;
use crate::scan;
use crate::scoring;

/// Show before/after score comparison.
pub fn show_diff(path: &std::path::Path) -> Result<()> {
    let before = score_project(path)?;

    let tmp = tempfile::TempDir::new()?;
    copy_dir_recursive(path, tmp.path())?;
    let fixes = apply_fixes_to_copy(tmp.path())?;

    let after = score_project(tmp.path())?;
    print_diff(&before, &after, &fixes);
    Ok(())
}

fn score_project(path: &std::path::Path) -> Result<scoring::Scorecard> {
    let mut ctx = scan::build_context(path)?;
    let pt = detection::detect(&ctx);
    ctx.project_type = Some(pt.clone());
    let results: Vec<_> = rules::all_rules().iter()
        .filter(|r| r.applies_to(&pt))
        .map(|r| r.check(&ctx))
        .collect();
    Ok(scoring::calculate(results, &pt))
}

fn apply_fixes_to_copy(path: &std::path::Path) -> Result<Vec<String>> {
    let mut ctx = scan::build_context(path)?;
    let pt = detection::detect(&ctx);
    ctx.project_type = Some(pt.clone());
    let results: Vec<_> = rules::all_rules().iter()
        .filter(|r| r.applies_to(&pt))
        .map(|r| r.check(&ctx))
        .collect();
    fix::apply_fixes(&ctx, &results)
}

fn print_diff(before: &scoring::Scorecard, after: &scoring::Scorecard, fixes: &[String]) {
    println!();
    println!("{}", "  Score Diff (current -> after --fix)".bold().underline());
    println!();

    for (b, a) in before.dimensions.iter().zip(after.dimensions.iter()) {
        let delta = a.score - b.score;
        let ds = format_delta(delta);
        println!("  {:<22} {:.0} -> {:.0}  ({})", b.dimension.to_string().bold(), b.score, a.score, ds);
    }

    println!();
    let td = after.total_score - before.total_score;
    println!("  {:<22} {:.0} -> {:.0}  ({})  {} -> {}", "TOTAL".bold(),
        before.total_score, after.total_score, format_delta(td), before.grade, after.grade);

    if !fixes.is_empty() {
        println!();
        println!("  {} to apply:", "Fixes available".green().bold());
        for f in fixes { println!("    {} {f}", "+".green()); }
        println!();
        println!("  Run {} to apply.", "claude-native --fix".bold());
    } else {
        println!();
        println!("  {} — no fixable issues.", "Already optimized".green());
    }
}

fn format_delta(delta: f64) -> String {
    if delta > 0.0 { format!("+{:.0}", delta).green().to_string() }
    else if delta < 0.0 { format!("{:.0}", delta).red().to_string() }
    else { "  0".dimmed().to_string() }
}

fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> Result<()> {
    for entry in walkdir::WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let rel = entry.path().strip_prefix(src)?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(p) = target.parent() { std::fs::create_dir_all(p)?; }
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
