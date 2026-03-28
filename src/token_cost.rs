use crate::rules::{RuleResult, Suggestion};
use crate::scan::ProjectContext;

/// Estimate token savings for a suggestion based on project state.
/// Returns a string like "~2000 tokens/search saved" or empty if not estimable.
pub fn estimate_savings(rule_id: &str, ctx: &ProjectContext) -> Option<String> {
    match rule_id {
        "1.1" => Some("Without CLAUDE.md, Claude wastes ~500-2000 tokens/session exploring your project from scratch".into()),
        "1.5" => {
            let noise_files = estimate_noise_files(ctx);
            if noise_files > 0 {
                Some(format!("~{noise_files} noisy files currently visible to Claude's Glob/Grep searches"))
            } else {
                None
            }
        }
        "2.1" => {
            let mega: Vec<_> = ctx.all_files.iter()
                .filter(|f| f.line_count > 500)
                .collect();
            let total_waste: usize = mega.iter().map(|f| f.line_count * 2).sum();
            if total_waste > 0 {
                Some(format!("~{total_waste} tokens wasted per read of these {}-file(s)", mega.len()))
            } else {
                None
            }
        }
        "2.3" => {
            let lock_lines: usize = ctx.lock_files.iter().map(|f| {
                crate::scan::file_stats::count_lines(f)
            }).sum();
            if lock_lines > 0 {
                Some(format!("Lock files total {lock_lines} lines = ~{} tokens if Claude reads them", lock_lines * 2))
            } else {
                None
            }
        }
        "4.2" => Some("Auto-format hooks save ~500 tokens/edit by preventing linter error → fix → re-check cycles".into()),
        "4.5" => Some("Each permission prompt interrupts Claude's flow, costing ~200 tokens of lost context".into()),
        "5.2" => Some("Without tests, Claude can't self-verify — leading to ~30% more revision cycles".into()),
        _ => None,
    }
}

fn estimate_noise_files(ctx: &ProjectContext) -> usize {
    // Estimate how many files would be excluded by a proper .claudeignore
    if ctx.claudeignore_content.is_some() { return 0; }
    let noise_dirs = ["node_modules", "target", ".venv", "vendor", "dist", "build"];
    ctx.directories.iter().filter(|d| {
        d.file_name()
            .and_then(|n| n.to_str())
            .map(|n| noise_dirs.contains(&n))
            .unwrap_or(false)
    }).count() * 100 // rough estimate: 100 files per noise dir
}

/// Enhance suggestions with token-cost estimates.
pub fn enrich_suggestions(suggestions: &mut [Suggestion], rule_results: &[RuleResult], ctx: &ProjectContext) {
    for suggestion in suggestions.iter_mut() {
        // Find the rule that generated this suggestion
        for r in rule_results {
            if let Some(ref s) = r.suggestion {
                if s.title == suggestion.title {
                    if let Some(estimate) = estimate_savings(&r.rule_id, ctx) {
                        suggestion.description = format!("{}\n\nToken impact: {estimate}", suggestion.description);
                    }
                    break;
                }
            }
        }
    }
}
