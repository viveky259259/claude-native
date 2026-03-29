use crate::rules::*;
use crate::scan::ProjectContext;

// ── Rule 3.4: Clear module boundaries ───────────────────────────────

pub struct ClearModuleBoundaries;

impl Rule for ClearModuleBoundaries {
    fn id(&self) -> &str { "3.4" }
    fn name(&self) -> &str { "Clear module boundaries" }
    fn dimension(&self) -> Dimension { Dimension::Navigation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let index_patterns = [
            "index.ts", "index.js", "index.tsx", "mod.rs",
            "__init__.py", "index.dart",
        ];

        let skip_dir_names = [".claude", ".github", "tests", "test", "spec", "examples", "docs"];
        let major_dirs: Vec<_> = ctx.directories.iter()
            .filter(|d| {
                if let Ok(rel) = d.strip_prefix(&ctx.root) {
                    let components: Vec<_> = rel.components().collect();
                    let dir_name = d.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    components.len() <= 2
                        && !dir_name.starts_with('.')
                        && !skip_dir_names.contains(&dir_name)
                        && ctx.all_files.iter().any(|f| f.path.starts_with(d) && !f.is_test)
                } else {
                    false
                }
            })
            .collect();

        if major_dirs.is_empty() {
            return self.pass();
        }

        let dirs_with_index = major_dirs.iter()
            .filter(|d| index_patterns.iter().any(|p| d.join(p).exists()))
            .count();

        let ratio = if major_dirs.is_empty() { 1.0 } else {
            dirs_with_index as f64 / major_dirs.len() as f64
        };

        if ratio >= 0.5 {
            self.pass()
        } else {
            self.warn(
                "Most directories lack clear module entry points (index/mod files)",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Add index/barrel files to modules".into(),
                    description: "Add index.ts/mod.rs/__init__.py to major directories. These let Claude understand what a module exports without reading every file.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule 3.5: Predictable test locations ────────────────────────────

pub struct PredictableTestLocations;

impl Rule for PredictableTestLocations {
    fn id(&self) -> &str { "3.5" }
    fn name(&self) -> &str { "Tests in predictable locations" }
    fn dimension(&self) -> Dimension { Dimension::Navigation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.test_files.is_empty() {
            return self.skip();
        }

        let mut co_located = 0;
        let mut in_test_dir = 0;

        for tf in &ctx.test_files {
            let path_str = tf.to_string_lossy();
            if path_str.contains("__tests__") || path_str.contains("/tests/") || path_str.contains("/test/") || path_str.contains("/spec/") {
                in_test_dir += 1;
            } else {
                co_located += 1;
            }
        }

        let total = co_located + in_test_dir;
        let max_pattern = co_located.max(in_test_dir);
        let consistency = max_pattern as f64 / total as f64;

        if consistency >= 0.7 {
            self.pass()
        } else {
            self.warn(
                "Tests are split between co-located and test directories — inconsistent pattern",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Standardize test locations".into(),
                    description: "Pick one pattern: either co-located (*.test.ts next to source) or centralized (tests/ directory). Consistency helps Claude find tests instantly.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule 3.6: No deep nesting ──────────────────────────────────────

pub struct NoDeepNesting;

impl Rule for NoDeepNesting {
    fn id(&self) -> &str { "3.6" }
    fn name(&self) -> &str { "No deeply nested directories (>4 levels)" }
    fn dimension(&self) -> Dimension { Dimension::Navigation }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.max_depth <= 4 {
            self.pass()
        } else {
            self.warn(
                &format!("Directory nesting depth is {} (target: ≤4)", ctx.max_depth),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Flatten directory structure".into(),
                    description: "Deep nesting makes Glob patterns expensive and navigation confusing. Consider flattening to ≤4 levels.".into(),
                    effort: Effort::HalfDay,
                },
            )
        }
    }
}

// ── Rule 3.7: Descriptive names ────────────────────────────────────

pub struct DescriptiveNames;

impl Rule for DescriptiveNames {
    fn id(&self) -> &str { "3.7" }
    fn name(&self) -> &str { "Descriptive directory and file names" }
    fn dimension(&self) -> Dimension { Dimension::Navigation }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let cryptic_dirs: Vec<String> = ctx.directories.iter()
            .filter_map(|d| {
                let name = d.file_name()?.to_str()?;
                let ok_short = ["db", "ui", "CI", "ci", "go", "js", "ts", "py"];
                if name.len() <= 2 && !ok_short.contains(&name) && !name.starts_with('.') {
                    Some(name.to_string())
                } else {
                    None
                }
            })
            .collect();

        if cryptic_dirs.is_empty() {
            self.pass()
        } else {
            self.warn(
                &format!("Cryptic directory names found: {}", cryptic_dirs.join(", ")),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Use descriptive directory names".into(),
                    description: format!("Rename these directories to something descriptive: {}. Claude uses names to decide what to read.", cryptic_dirs.join(", ")),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule 3.8: Folder-level CLAUDE.md for navigation ─────────────────

pub struct FolderClaudeMds;

impl Rule for FolderClaudeMds {
    fn id(&self) -> &str { "3.8" }
    fn name(&self) -> &str { "Folders have CLAUDE.md for navigation" }
    fn dimension(&self) -> Dimension { Dimension::Navigation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        // Only check source directories with 3+ source files
        let source_dirs = find_source_dirs(ctx);
        if source_dirs.is_empty() {
            return self.pass();
        }

        let dirs_with_claude_md: Vec<_> = source_dirs.iter()
            .filter(|d| d.join("CLAUDE.md").exists())
            .collect();

        let dirs_without: Vec<String> = source_dirs.iter()
            .filter(|d| !d.join("CLAUDE.md").exists())
            .filter_map(|d| d.strip_prefix(&ctx.root).ok())
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        let ratio = dirs_with_claude_md.len() as f64 / source_dirs.len() as f64;

        if ratio >= 0.7 {
            self.pass()
        } else if dirs_without.len() <= 2 {
            self.warn(
                &format!("{} folder(s) lack a CLAUDE.md: {}", dirs_without.len(), dirs_without.join(", ")),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Add CLAUDE.md to source folders".into(),
                    description: format!(
                        "Add a small CLAUDE.md (3-5 lines) to each folder explaining what it contains. \
                         This lets Claude understand a folder's purpose by reading one file instead of scanning all files.\n\
                         Missing in: {}", dirs_without.join(", ")
                    ),
                    effort: Effort::Minutes,
                },
            )
        } else {
            self.fail(
                &format!("{}/{} source folders lack CLAUDE.md", dirs_without.len(), source_dirs.len()),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Add CLAUDE.md to source folders".into(),
                    description: format!(
                        "Add a small CLAUDE.md (3-5 lines) to each source folder. Example:\n\
                         ```\n\
                         # rules/\n\
                         Rule implementations. Each file = one scoring dimension.\n\
                         All rules implement the `Rule` trait from mod.rs.\n\
                         ```\n\
                         This saves Claude ~500 tokens per folder by avoiding full file scans.\n\
                         Missing in: {}", dirs_without.iter().take(5).cloned().collect::<Vec<_>>().join(", ")
                    ),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

/// Find directories that contain 3+ source files (worth having a CLAUDE.md).
fn find_source_dirs(ctx: &ProjectContext) -> Vec<std::path::PathBuf> {
    use std::collections::HashMap;

    let skip = [".claude", ".github", "tests", "test", "docs", "examples"];
    let mut dir_counts: HashMap<std::path::PathBuf, usize> = HashMap::new();

    for f in ctx.source_files() {
        if let Some(parent) = f.path.parent() {
            // Skip root and non-source dirs
            if parent == ctx.root { continue; }
            let dir_name = parent.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if dir_name.starts_with('.') || skip.contains(&dir_name) { continue; }
            *dir_counts.entry(parent.to_path_buf()).or_insert(0) += 1;
        }
    }

    dir_counts.into_iter()
        .filter(|(_, count)| *count >= 3)
        .map(|(path, _)| path)
        .collect()
}
