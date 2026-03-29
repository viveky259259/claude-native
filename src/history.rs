use std::fs;
use std::path::Path;
use std::time::SystemTime;

use anyhow::Result;
use owo_colors::OwoColorize;
use serde_json::{json, Value};

use crate::scoring::Scorecard;

const HISTORY_FILE: &str = ".claude-native-history.json";

/// Append current score to history file and display trend.
pub fn record_and_show(sc: &Scorecard, root: &Path) -> Result<()> {
    let path = root.join(HISTORY_FILE);
    let mut entries = load_history(&path);

    let entry = json!({
        "timestamp": unix_timestamp(),
        "score": sc.total_score,
        "grade": format!("{}", sc.grade),
        "project_type": format!("{}", sc.project_type),
        "dimensions": sc.dimensions.iter().map(|d| json!({
            "name": format!("{}", d.dimension),
            "score": d.score,
        })).collect::<Vec<_>>(),
    });

    entries.push(entry);
    fs::write(&path, serde_json::to_string_pretty(&entries)?)?;

    print_trend(&entries);
    Ok(())
}

fn load_history(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn print_trend(entries: &[Value]) {
    println!("{}", "  Score History".bold().underline());
    println!();

    let show = entries.len().min(10);
    let start = entries.len().saturating_sub(show);

    for (i, entry) in entries[start..].iter().enumerate() {
        let score = entry.get("score").and_then(|s| s.as_f64()).unwrap_or(0.0);
        let grade = entry.get("grade").and_then(|g| g.as_str()).unwrap_or("?");
        let ts = entry.get("timestamp").and_then(|t| t.as_u64()).unwrap_or(0);

        let delta_str = if i > 0 || start > 0 {
            let prev_idx = if i > 0 { start + i - 1 } else { start.saturating_sub(1) };
            let prev = entries.get(prev_idx)
                .and_then(|e| e.get("score"))
                .and_then(|s| s.as_f64())
                .unwrap_or(score);
            let delta = score - prev;
            if delta > 0.0 { format!(" {}", format!("+{:.0}", delta).green()) }
            else if delta < 0.0 { format!(" {}", format!("{:.0}", delta).red()) }
            else { String::new() }
        } else {
            String::new()
        };

        let date = format_timestamp(ts);
        let grade_colored = match grade {
            "A+" | "A" => grade.green().bold().to_string(),
            "B" => grade.yellow().bold().to_string(),
            _ => grade.red().to_string(),
        };

        println!("  {} {} {:.0}/100{}", date.dimmed(), grade_colored, score, delta_str);
    }

    println!();
    println!("  {} entries total. History: {}", entries.len(), HISTORY_FILE.dimmed());
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn format_timestamp(ts: u64) -> String {
    let secs = ts % 60;
    let mins = (ts / 60) % 60;
    let hours = (ts / 3600) % 24;
    let days = ts / 86400;
    let years = 1970 + days / 365;
    let remaining_days = days % 365;
    let month = remaining_days / 30 + 1;
    let day = remaining_days % 30 + 1;
    format!("{years}-{month:02}-{day:02} {hours:02}:{mins:02}:{secs:02}")
}
