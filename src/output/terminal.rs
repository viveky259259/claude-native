use owo_colors::OwoColorize;

use crate::rules::{RuleStatus, Severity, SuggestionPriority};
use crate::scoring::{Grade, Scorecard};

pub fn render(sc: &Scorecard) {
    println!();
    print_header(sc);
    println!();
    print_dimensions(sc);
    println!();
    print_rule_details(sc);
    println!();
    print_suggestions(sc);
    println!();
}

fn print_header(sc: &Scorecard) {
    let grade_colored = match sc.grade {
        Grade::APlus | Grade::A => format!("{}", sc.grade).green().bold().to_string(),
        Grade::B => format!("{}", sc.grade).yellow().bold().to_string(),
        Grade::C => format!("{}", sc.grade).yellow().to_string(),
        Grade::D => format!("{}", sc.grade).red().to_string(),
        Grade::F => format!("{}", sc.grade).red().bold().to_string(),
    };

    let score_colored = match sc.grade {
        Grade::APlus | Grade::A => format!("{:.0}", sc.total_score).green().bold().to_string(),
        Grade::B => format!("{:.0}", sc.total_score).yellow().bold().to_string(),
        Grade::C => format!("{:.0}", sc.total_score).yellow().to_string(),
        Grade::D => format!("{:.0}", sc.total_score).red().to_string(),
        Grade::F => format!("{:.0}", sc.total_score).red().bold().to_string(),
    };

    println!("{}", "╔══════════════════════════════════════════════════════════╗".dimmed());
    println!("{}  {}  {}", "║".dimmed(), "Claude Native Score".bold(), "║".dimmed());
    println!("{}", "╠══════════════════════════════════════════════════════════╣".dimmed());
    println!("{}  Project Type: {:<40} {}", "║".dimmed(), sc.project_type, "║".dimmed());
    println!("{}  Score: {}/100  Grade: {:<33} {}", "║".dimmed(), score_colored, grade_colored, "║".dimmed());
    println!("{}  {:<54} {}", "║".dimmed(), sc.grade.description(), "║".dimmed());
    println!("{}", "╚══════════════════════════════════════════════════════════╝".dimmed());
}

fn print_dimensions(sc: &Scorecard) {
    println!("{}", "  Dimension Scores".bold().underline());
    println!();

    for ds in &sc.dimensions {
        let bar_width = 30;
        let filled = ((ds.score / 100.0) * bar_width as f64) as usize;
        let empty = bar_width - filled;

        let bar_char = "█";
        let empty_char = "░";

        let bar = if ds.score >= 80.0 {
            format!("{}{}", bar_char.repeat(filled).green(), empty_char.repeat(empty).dimmed())
        } else if ds.score >= 60.0 {
            format!("{}{}", bar_char.repeat(filled).yellow(), empty_char.repeat(empty).dimmed())
        } else {
            format!("{}{}", bar_char.repeat(filled).red(), empty_char.repeat(empty).dimmed())
        };

        let capped_mark = if ds.capped { " CAPPED".red().bold().to_string() } else { String::new() };

        println!(
            "  {:<22} {} {:.0}/100  (w: {:.0}%){}",
            ds.dimension.to_string().bold(),
            bar,
            ds.score,
            ds.weight * 100.0,
            capped_mark,
        );

        println!(
            "  {:<22} {} passed, {} failed, {} warned, {} skipped",
            "",
            ds.rules_passed.to_string().green(),
            ds.rules_failed.to_string().red(),
            ds.rules_warned.to_string().yellow(),
            ds.rules_skipped.to_string().dimmed(),
        );
    }
}

fn print_rule_details(sc: &Scorecard) {
    println!("{}", "  Rule Results".bold().underline());
    println!();

    for r in &sc.rule_results {
        let icon = match &r.status {
            RuleStatus::Pass => "✓".green().to_string(),
            RuleStatus::Warn(_) => "⚠".yellow().to_string(),
            RuleStatus::Fail(_) => "✗".red().to_string(),
            RuleStatus::Skip => "○".dimmed().to_string(),
        };

        let severity = match r.severity {
            Severity::Critical => r.severity.to_string().red().bold().to_string(),
            Severity::High => r.severity.to_string().red().to_string(),
            Severity::Medium => r.severity.to_string().yellow().to_string(),
            Severity::Low => r.severity.to_string().dimmed().to_string(),
        };

        let msg = match &r.status {
            RuleStatus::Pass => "".to_string(),
            RuleStatus::Warn(m) => format!(" — {m}"),
            RuleStatus::Fail(m) => format!(" — {m}"),
            RuleStatus::Skip => " (skipped)".dimmed().to_string(),
        };

        // Truncate message for display
        let msg_display = if msg.len() > 80 {
            format!("{}...", &msg[..77])
        } else {
            msg
        };

        println!(
            "  {} [{:>4}] {:<44} {}{}",
            icon,
            r.rule_id,
            r.name,
            severity,
            msg_display,
        );
    }
}

fn print_suggestions(sc: &Scorecard) {
    if sc.suggestions.is_empty() {
        println!("  {} No suggestions — your project is well-configured!", "🎉".green());
        return;
    }

    println!("{}", "  Suggestions (prioritized)".bold().underline());
    println!();

    let mut current_priority: Option<SuggestionPriority> = None;

    for (i, sug) in sc.suggestions.iter().enumerate() {
        if current_priority.as_ref() != Some(&sug.priority) {
            current_priority = Some(sug.priority);
            let header = match sug.priority {
                SuggestionPriority::QuickWin => "Quick Wins (< 2 minutes)".green().bold().to_string(),
                SuggestionPriority::HighImpact => "High Impact".yellow().bold().to_string(),
                SuggestionPriority::NiceToHave => "Nice to Have".dimmed().to_string(),
            };
            println!("  {header}");
        }

        println!(
            "    {}. {} {}",
            (i + 1).to_string().bold(),
            sug.title.bold(),
            format!("[{}]", sug.effort).dimmed(),
        );
        // Print description indented, wrapping at ~70 chars
        for line in wrap_text(&sug.description, 68) {
            println!("       {}", line.dimmed());
        }
    }
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for paragraph in text.lines() {
        if paragraph.is_empty() {
            lines.push(String::new());
            continue;
        }
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        let mut current_line = String::new();
        for word in words {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
    }
    lines
}
