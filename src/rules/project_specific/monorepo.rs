use crate::detection::{PrimaryType, ProjectType};
use crate::rules::*;
use crate::scan::ProjectContext;

// ── Rule M1: Root CLAUDE.md must be thin orchestrator ───────────────

pub struct RootClaudeMdThin;

impl Rule for RootClaudeMdThin {
    fn id(&self) -> &str { "M1" }
    fn name(&self) -> &str { "Root CLAUDE.md is thin (<100 lines)" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool {
        matches!(pt.primary, PrimaryType::Monorepo)
    }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let lines = ctx.claude_md_line_count();
        if lines == 0 {
            return self.skip();
        }
        if lines <= 100 {
            self.pass()
        } else {
            self.fail(
                &format!("Root CLAUDE.md is {lines} lines (monorepo target: <100). Package-specific instructions should be in subdirectory CLAUDE.md files."),
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Slim root CLAUDE.md to <100 lines".into(),
                    description: "In a monorepo, root CLAUDE.md loads on EVERY request across ALL packages. Move package-specific instructions to packages/<name>/CLAUDE.md.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule M2: Every package has its own CLAUDE.md ────────────────────

pub struct PerPackageClaudeMd;

impl Rule for PerPackageClaudeMd {
    fn id(&self) -> &str { "M2" }
    fn name(&self) -> &str { "Per-package CLAUDE.md files" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool {
        matches!(pt.primary, PrimaryType::Monorepo)
    }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let workspace_dirs = ["packages", "apps", "services", "libs"];
        let mut total_packages = 0;
        let mut packages_with_claude_md = 0;

        for wd in &workspace_dirs {
            let dir = ctx.root.join(wd);
            if !dir.is_dir() { continue; }
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        total_packages += 1;
                        if entry.path().join("CLAUDE.md").exists()
                            || entry.path().join(".claude").join("CLAUDE.md").exists()
                        {
                            packages_with_claude_md += 1;
                        }
                    }
                }
            }
        }

        if total_packages == 0 {
            return self.skip();
        }

        if packages_with_claude_md == total_packages {
            self.pass()
        } else {
            let missing = total_packages - packages_with_claude_md;
            self.fail(
                &format!("{missing}/{total_packages} packages lack their own CLAUDE.md"),
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: format!("Add CLAUDE.md to {missing} packages"),
                    description: "Each package should have its own CLAUDE.md with package-specific build/test commands and conventions. These load on-demand, saving tokens.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule M3: .claudeignore covers all workspace outputs ─────────────

pub struct WorkspaceOutputsIgnored;

impl Rule for WorkspaceOutputsIgnored {
    fn id(&self) -> &str { "M3" }
    fn name(&self) -> &str { ".claudeignore covers all workspace outputs" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool {
        matches!(pt.primary, PrimaryType::Monorepo)
    }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.claudeignore_content.is_none() {
            return self.fail(
                "No .claudeignore in monorepo — all workspace outputs are visible",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Create .claudeignore with workspace globs".into(),
                    description: "Add patterns like: packages/*/dist/, apps/*/build/, **/node_modules/, **/target/. Monorepos multiply noise — 10 packages x unignored dist/ = 10x wasted tokens.".into(),
                    effort: Effort::Minutes,
                },
            );
        }

        let has_wildcard_patterns = ctx.claudeignore_contains("*/dist")
            || ctx.claudeignore_contains("*/build")
            || ctx.claudeignore_contains("**/dist")
            || ctx.claudeignore_contains("**/build")
            || ctx.claudeignore_contains("**/target");

        if has_wildcard_patterns {
            self.pass()
        } else {
            self.warn(
                ".claudeignore may not cover all workspace package outputs",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Add workspace-wide ignore patterns".into(),
                    description: "Use glob patterns: packages/*/dist/, apps/*/build/, **/node_modules/. These cover all current and future packages.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule M4: Path-scoped rules per package ──────────────────────────

pub struct PathScopedRulesPerPackage;

impl Rule for PathScopedRulesPerPackage {
    fn id(&self) -> &str { "M4" }
    fn name(&self) -> &str { "Path-scoped rules per package" }
    fn dimension(&self) -> Dimension { Dimension::Tooling }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool {
        matches!(pt.primary, PrimaryType::Monorepo)
    }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.has_claude_rules_dir {
            self.pass()
        } else {
            self.fail(
                "No .claude/rules/ directory in monorepo",
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Create path-scoped rules for packages".into(),
                    description: "Create .claude/rules/ with files scoped to packages via paths: frontmatter. A React rule shouldn't load when editing a Go backend.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule M5: Per-package test commands documented ───────────────────

pub struct PerPackageTestCommands;

impl Rule for PerPackageTestCommands {
    fn id(&self) -> &str { "M5" }
    fn name(&self) -> &str { "Per-package test commands documented" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool {
        matches!(pt.primary, PrimaryType::Monorepo)
    }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        // Check if root CLAUDE.md or any subdirectory CLAUDE.md mentions per-package testing
        let root_has_package_tests = ctx.claude_md_content.as_ref().map(|c| {
            let lower = c.to_lowercase();
            (lower.contains("cd ") && lower.contains("test"))
                || lower.contains("package")
                || lower.contains("workspace")
        }).unwrap_or(false);

        let subdir_count = ctx.subdirectory_claude_mds.len();

        if root_has_package_tests || subdir_count >= 2 {
            self.pass()
        } else {
            self.fail(
                "No per-package test commands found",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Document per-package test commands".into(),
                    description: "Running ALL tests after a single-package change wastes minutes. Document per-package commands: `cd packages/auth && npm test` or put them in per-package CLAUDE.md files.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule M6: Workspace dependency graph is navigable ────────────────

pub struct WorkspaceDepsNavigable;

impl Rule for WorkspaceDepsNavigable {
    fn id(&self) -> &str { "M6" }
    fn name(&self) -> &str { "Workspace dependencies are explicit" }
    fn dimension(&self) -> Dimension { Dimension::Navigation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool {
        matches!(pt.primary, PrimaryType::Monorepo)
    }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        // Check if workspace references are used in package.json or Cargo.toml
        let has_workspace_refs = ctx.package_manifests.iter().any(|m| {
            if let Ok(content) = std::fs::read_to_string(&m.path) {
                content.contains("workspace:") || content.contains("workspace = true")
                    || content.contains("\"link:") || content.contains("\"file:")
            } else {
                false
            }
        });

        if has_workspace_refs || ctx.package_manifests.len() <= 1 {
            self.pass()
        } else {
            self.warn(
                "No workspace dependency references found between packages",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Use workspace dependency references".into(),
                    description: "Use workspace:* (npm/pnpm), path dependencies (Cargo), or replace directives (Go) for inter-package deps. This lets Claude trace cross-package imports.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule M7: Shared tooling config at root ──────────────────────────

pub struct SharedToolingConfig;

impl Rule for SharedToolingConfig {
    fn id(&self) -> &str { "M7" }
    fn name(&self) -> &str { "Shared tooling config at root" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::Low }

    fn applies_to(&self, pt: &ProjectType) -> bool {
        matches!(pt.primary, PrimaryType::Monorepo)
    }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let shared_configs = [
            "tsconfig.json", "tsconfig.base.json",
            ".eslintrc", ".eslintrc.js", ".eslintrc.json", "eslint.config.js",
            "prettier.config.js", ".prettierrc",
            "rustfmt.toml", ".rustfmt.toml",
            ".editorconfig",
        ];

        let found = shared_configs.iter().any(|c| ctx.has_file(c));
        if found {
            self.pass()
        } else {
            self.warn(
                "No shared tooling configs found at monorepo root",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Add shared configs at root".into(),
                    description: "Place shared configs (tsconfig.base.json, .eslintrc, rustfmt.toml) at root for packages to extend. This ensures consistency Claude can rely on.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}
