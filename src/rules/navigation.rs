use std::collections::HashMap;

use crate::rules::*;
use crate::scan::ProjectContext;

// ── Rule 3.1: Clear directory structure ─────────────────────────────

pub struct ClearDirectoryStructure;

impl Rule for ClearDirectoryStructure {
    fn id(&self) -> &str { "3.1" }
    fn name(&self) -> &str { "Clear directory structure" }
    fn dimension(&self) -> Dimension { Dimension::Navigation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let root_source_files = ctx.all_files.iter()
            .filter(|f| {
                f.relative_path.parent().map(|p| p == std::path::Path::new("")).unwrap_or(true)
                    && !f.is_test
            })
            .count();

        if root_source_files > 15 {
            return self.fail(
                &format!("{root_source_files} source files at project root (>15)"),
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Organize root files into directories".into(),
                    description: "Move source files into logical directories (src/, lib/, utils/).".into(),
                    effort: Effort::HalfDay,
                },
            );
        }

        let crowded = find_crowded_dirs(ctx);
        if !crowded.is_empty() {
            self.warn(
                &format!("Directories with >15 source files:\n{}", crowded.join("\n")),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Split crowded directories".into(),
                    description: format!("Consider sub-grouping:\n{}", crowded.join("\n")),
                    effort: Effort::Hour,
                },
            )
        } else {
            self.pass()
        }
    }
}

fn find_crowded_dirs(ctx: &ProjectContext) -> Vec<String> {
    let mut dir_counts: HashMap<String, usize> = HashMap::new();
    for f in &ctx.all_files {
        if !f.is_test && !f.is_generated {
            let dir = f.relative_path.parent()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            *dir_counts.entry(dir).or_insert(0) += 1;
        }
    }
    dir_counts.iter()
        .filter(|(_, &count)| count > 15)
        .map(|(dir, count)| format!("  {dir}/ ({count} files)"))
        .collect()
}

// ── Rule 3.2: Consistent naming ────────────────────────────────────

pub struct ConsistentNaming;

const SKIP_NAMES: &[&str] = &[
    "Makefile", "Dockerfile", "Gemfile", "Rakefile", "README",
    "CLAUDE", "CHANGELOG", "LICENSE", "CONTRIBUTING", "GOLDEN",
    "Cargo", "Pipfile", "Procfile", "Vagrantfile", "MEMORY",
];

impl Rule for ConsistentNaming {
    fn id(&self) -> &str { "3.2" }
    fn name(&self) -> &str { "Consistent file naming conventions" }
    fn dimension(&self) -> Dimension { Dimension::Navigation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let (counts, total) = count_naming_conventions(ctx);
        if total < 5 {
            return self.pass();
        }
        let max_convention = counts.iter().max().copied().unwrap_or(0);
        let consistency = max_convention as f64 / total as f64;
        naming_result(self, consistency)
    }
}

fn count_naming_conventions(ctx: &ProjectContext) -> ([usize; 4], usize) {
    let mut counts = [0usize; 4]; // snake, kebab, camel, pascal
    let mut total = 0;

    for f in &ctx.all_files {
        let stem = match f.path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => continue,
        };
        if should_skip_name(stem) { continue; }
        total += 1;
        counts[classify_name(stem)] += 1;
    }
    (counts, total)
}

fn should_skip_name(stem: &str) -> bool {
    SKIP_NAMES.iter().any(|s| stem.starts_with(s))
        || stem.starts_with('.')
        || stem.len() < 2
        || (stem == stem.to_uppercase() && !stem.contains('-'))
}

fn classify_name(stem: &str) -> usize {
    let is_lower = stem == stem.to_lowercase();
    let has_underscore = stem.contains('_');
    let has_dash = stem.contains('-');
    let has_upper = stem.chars().any(|c| c.is_uppercase());

    if (has_underscore && is_lower) || (is_lower && !has_underscore && !has_dash) {
        0 // snake_case (includes single-word lowercase)
    } else if has_dash && is_lower {
        1 // kebab-case
    } else if stem.chars().next().map(|c| c.is_lowercase()).unwrap_or(false) && has_upper {
        2 // camelCase
    } else if stem.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        3 // PascalCase
    } else {
        0 // default to snake
    }
}

fn naming_result(rule: &ConsistentNaming, consistency: f64) -> RuleResult {
    if consistency >= 0.8 {
        rule.pass()
    } else if consistency >= 0.6 {
        rule.warn(
            &format!("File naming is ~{:.0}% consistent (target: >80%)", consistency * 100.0),
            Suggestion {
                priority: SuggestionPriority::NiceToHave,
                title: "Standardize file naming".into(),
                description: "Pick one convention and apply consistently.".into(),
                effort: Effort::Hour,
            },
        )
    } else {
        rule.fail(
            &format!("File naming is only ~{:.0}% consistent", consistency * 100.0),
            Suggestion {
                priority: SuggestionPriority::NiceToHave,
                title: "Standardize file naming".into(),
                description: "Mixed naming conventions force Claude to search instead of predict.".into(),
                effort: Effort::HalfDay,
            },
        )
    }
}

// ── Rule 3.3: Obvious entry points ─────────────────────────────────

pub struct ObviousEntryPoints;

impl Rule for ObviousEntryPoints {
    fn id(&self) -> &str { "3.3" }
    fn name(&self) -> &str { "Entry points are obvious" }
    fn dimension(&self) -> Dimension { Dimension::Navigation }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let entry_names = [
            "main.rs", "main.go", "main.py", "main.dart", "main.ts", "main.js",
            "index.ts", "index.js", "index.tsx", "index.jsx",
            "app.py", "app.ts", "app.js", "app.rb",
            "manage.py", "server.ts", "server.js", "lib.rs", "mod.rs",
        ];
        let has_entry = ctx.all_files.iter().any(|f| {
            f.path.file_name().and_then(|n| n.to_str())
                .map(|n| entry_names.contains(&n)).unwrap_or(false)
        });
        let documented = ctx.claude_md_content.as_ref().map(|c| {
            let l = c.to_lowercase();
            l.contains("entry point") || l.contains("entrypoint")
        }).unwrap_or(false);

        if has_entry || documented {
            self.pass()
        } else {
            self.warn(
                "No obvious entry point found (main.*, index.*, app.*)",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Document entry point in CLAUDE.md".into(),
                    description: "Add 'Entry point: src/server.ts' to CLAUDE.md.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}
