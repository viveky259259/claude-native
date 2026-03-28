use std::path::Path;

use crate::rules::*;
use crate::scan::ProjectContext;

/// Check if the longest function in a file is a simple registry/list
/// (mostly Box::new, vec![], or simple match arms). These are long
/// but not complex — they don't hurt Claude's ability to understand.
fn is_registry_function(path: &Path, _longest: usize) -> bool {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    let total_lines = content.lines().count();
    if total_lines == 0 { return false; }
    let simple_lines = content.lines().filter(|l| {
        let t = l.trim();
        t.starts_with("Box::new(") || t.starts_with("rules.push(")
            || t.starts_with("vec![") || t.starts_with("]")
            || t.starts_with("Some(") || t.starts_with("None")
            || t.starts_with("if ") || t.starts_with("} else")
            || t.starts_with("match ") || t.starts_with("=>")
            || t.starts_with("let ") || t.starts_with("pub ")
            || t.is_empty() || t.starts_with("//") || t.starts_with("use ")
            || t == "}" || t == "{" || t.starts_with("return ")
    }).count();
    simple_lines as f64 / total_lines as f64 > 0.7
}

// ── Rule 2.1: No mega-files ────────────────────────────────────────

pub struct NoMegaFiles;

impl Rule for NoMegaFiles {
    fn id(&self) -> &str { "2.1" }
    fn name(&self) -> &str { "No mega-files (>500 lines)" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::High }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let threshold = 500;
        let warn_threshold = 300;

        let mega = ctx.mega_files(threshold);
        let large = ctx.all_files.iter()
            .filter(|f| f.line_count > warn_threshold && f.line_count <= threshold && !f.is_test && !f.is_generated)
            .count();

        if mega.is_empty() && large == 0 {
            return self.pass();
        }

        if mega.is_empty() {
            return self.warn(
                &format!("{large} files exceed {warn_threshold} lines (approaching the {threshold}-line limit)"),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Consider splitting large files".into(),
                    description: format!("{large} files are between {warn_threshold}-{threshold} lines. Smaller files = cheaper reads for Claude. Consider splitting by concern."),
                    effort: Effort::Hour,
                },
            );
        }

        let count = mega.len();
        let examples: Vec<String> = mega.iter()
            .take(3)
            .map(|f| format!("  {} ({} lines)", f.relative_path.display(), f.line_count))
            .collect();

        self.fail(
            &format!("{count} files exceed {threshold} lines:\n{}", examples.join("\n")),
            Suggestion {
                priority: SuggestionPriority::HighImpact,
                title: format!("Split {count} mega-file(s)"),
                description: format!("Files over {threshold} lines cost Claude ~{} tokens per read even if only 10 lines are relevant. Split by responsibility.\n{}", threshold * 2, examples.join("\n")),
                effort: Effort::HalfDay,
            },
        )
    }
}

// ── Rule 2.2: No mega-functions ────────────────────────────────────

pub struct NoMegaFunctions;

impl Rule for NoMegaFunctions {
    fn id(&self) -> &str { "2.2" }
    fn name(&self) -> &str { "No mega-functions (>80 lines)" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        use crate::scan::file_stats;

        let threshold = 80;
        let warn_threshold = 50;
        let mut worst_file = String::new();
        let mut worst_len = 0;
        let mut offending_count = 0;

        for f in &ctx.all_files {
            if f.is_test || f.is_generated || f.line_count < warn_threshold {
                continue;
            }
            if ctx.is_claudeignored(&f.relative_path.to_string_lossy()) {
                continue;
            }
            let (longest, _count) = file_stats::longest_function(&f.path);
            // Skip registry/list functions (just Box::new, vec![], match arms)
            if longest > threshold && !is_registry_function(&f.path, longest) {
                offending_count += 1;
                if longest > worst_len {
                    worst_len = longest;
                    worst_file = f.relative_path.to_string_lossy().to_string();
                }
            }
        }

        if offending_count == 0 {
            self.pass()
        } else {
            self.warn(
                &format!("{offending_count} file(s) contain functions >80 lines (worst: {worst_file} at {worst_len} lines)"),
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Break down large functions".into(),
                    description: format!("Functions over 80 lines force Claude to read more context. Split by responsibility. Worst offender: {worst_file} ({worst_len} lines)."),
                    effort: Effort::HalfDay,
                },
            )
        }
    }
}

// ── Rule 2.3: Lock files are ignored ────────────────────────────────

pub struct LockFilesIgnored;

impl Rule for LockFilesIgnored {
    fn id(&self) -> &str { "2.3" }
    fn name(&self) -> &str { "Lock files are in .claudeignore" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::High }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.lock_files.is_empty() {
            return self.pass(); // No lock files to worry about
        }

        if ctx.claudeignore_content.is_none() {
            return self.fail(
                "Lock files exist but no .claudeignore to exclude them",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Add lock files to .claudeignore".into(),
                    description: "Add these to .claudeignore: package-lock.json, yarn.lock, Cargo.lock, Gemfile.lock, poetry.lock, go.sum. Lock files can be 10,000+ lines with zero useful context.".into(),
                    effort: Effort::Minutes,
                },
            );
        }

        let lock_patterns = [
            "package-lock", "yarn.lock", "pnpm-lock", "Cargo.lock",
            "Gemfile.lock", "poetry.lock", "go.sum", "composer.lock",
            "pubspec.lock", "Pipfile.lock",
        ];

        let missing: Vec<&str> = ctx.lock_files.iter()
            .filter_map(|lf| {
                let name = lf.file_name()?.to_str()?;
                let is_covered = lock_patterns.iter().any(|p| {
                    ctx.claudeignore_contains(p)
                });
                if is_covered { None } else { Some(name) }
            })
            .collect();

        if missing.is_empty() {
            self.pass()
        } else {
            self.fail(
                &format!("Lock files not in .claudeignore: {}", missing.join(", ")),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Add lock files to .claudeignore".into(),
                    description: format!("Add these patterns to .claudeignore: {}", missing.join(", ")),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule 2.4: Generated files are ignored ───────────────────────────

pub struct GeneratedFilesIgnored;

impl Rule for GeneratedFilesIgnored {
    fn id(&self) -> &str { "2.4" }
    fn name(&self) -> &str { "Generated/compiled files are ignored" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::High }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let generated_count = ctx.all_files.iter().filter(|f| f.is_generated).count();

        if generated_count == 0 {
            return self.pass();
        }

        let build_dirs = ["dist", "build", "target", ".next", "out", "coverage"];
        let has_build_dirs_ignored = ctx.claudeignore_content.as_ref().map(|c| {
            build_dirs.iter().any(|d| c.contains(d))
        }).unwrap_or(false);

        if has_build_dirs_ignored {
            self.pass()
        } else {
            self.fail(
                &format!("{generated_count} generated files found but build directories not in .claudeignore"),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Exclude build/generated directories".into(),
                    description: "Add to .claudeignore: dist/, build/, target/, .next/, out/, coverage/, **/generated/. Generated files waste tokens and can confuse Claude.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// Rules 2.5-2.7 are in context_extra.rs
