use std::path::Path;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use owo_colors::OwoColorize;

use crate::detection;
use crate::rules;
use crate::scan;
use crate::scoring;

/// Watch for file changes and re-score automatically.
/// Uses polling (checks every 2 seconds) to avoid extra dependencies.
pub fn watch_and_score(path: &Path) -> Result<()> {
    println!("{}", "Watching for changes... (Ctrl+C to stop)".dimmed());
    println!();

    let mut last_score = 0.0_f64;
    let mut last_mod = get_latest_mtime(path);

    loop {
        let current_mod = get_latest_mtime(path);

        if current_mod != last_mod || last_score == 0.0 {
            last_mod = current_mod;

            match run_score(path) {
                Ok((score, grade, pt)) => {
                    let delta = score - last_score;
                    let delta_str = if last_score == 0.0 {
                        String::new()
                    } else if delta > 0.0 {
                        format!(" ({})", format!("+{:.0}", delta).green())
                    } else if delta < 0.0 {
                        format!(" ({})", format!("{:.0}", delta).red())
                    } else {
                        String::new()
                    };

                    let now = chrono_now();
                    let grade_colored = match grade.as_str() {
                        "A+" | "A" => grade.green().bold().to_string(),
                        "B" => grade.yellow().bold().to_string(),
                        _ => grade.red().to_string(),
                    };

                    println!(
                        "  {} {} {}/100 {}{} — {}",
                        now.dimmed(),
                        grade_colored,
                        format!("{:.0}", score).bold(),
                        pt.dimmed(),
                        delta_str,
                        if score >= 90.0 { "Claude Native".green().to_string() }
                        else if score >= 70.0 { "Claude Friendly".yellow().to_string() }
                        else { "Needs work".red().to_string() },
                    );

                    last_score = score;
                }
                Err(e) => {
                    eprintln!("  {} {e}", "Error:".red());
                }
            }
        }

        std::thread::sleep(Duration::from_secs(2));
    }
}

fn run_score(path: &Path) -> Result<(f64, String, String)> {
    let mut ctx = scan::build_context(path)?;
    let pt = detection::detect(&ctx);
    ctx.project_type = Some(pt.clone());

    let all = rules::all_rules();
    let results: Vec<_> = all.iter()
        .filter(|r| r.applies_to(&pt))
        .map(|r| r.check(&ctx))
        .collect();

    let sc = scoring::calculate(results, &pt);
    Ok((sc.total_score, format!("{}", sc.grade), format!("{}", pt)))
}

fn get_latest_mtime(path: &Path) -> u64 {
    let mut latest = 0u64;
    if let Ok(walker) = walkdir::WalkDir::new(path)
        .max_depth(4)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        for entry in walker {
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    let secs = modified.duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default().as_secs();
                    if secs > latest { latest = secs; }
                }
            }
        }
    }
    latest
}

fn chrono_now() -> String {
    // Simple timestamp without chrono dependency
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let secs = now % 60;
    let mins = (now / 60) % 60;
    let hours = (now / 3600) % 24;
    format!("{hours:02}:{mins:02}:{secs:02}")
}
