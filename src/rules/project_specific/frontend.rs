use crate::detection::{PrimaryType, ProjectType};
use crate::rules::*;
use crate::scan::ProjectContext;

fn is_frontend(pt: &ProjectType) -> bool {
    matches!(pt.primary, PrimaryType::Frontend(_))
}

// ── Rule FE1: Build cache directories ignored ───────────────────────

pub struct BuildCacheIgnored;

impl Rule for BuildCacheIgnored {
    fn id(&self) -> &str { "FE1" }
    fn name(&self) -> &str { "Build cache directories ignored" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Critical }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_frontend(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let caches = [".next", ".nuxt", ".angular", "dist", "build", ".vercel", ".turbo"];
        if ctx.claudeignore_content.is_none() {
            return self.fail(
                "No .claudeignore — framework build caches (.next/ can be 500MB+) are visible",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Create .claudeignore for frontend project".into(),
                    description: format!("Add to .claudeignore: {}, node_modules/, coverage/", caches.join("/, ") + "/"),
                    effort: Effort::Minutes,
                },
            );
        }

        let missing: Vec<&&str> = caches.iter()
            .filter(|c| ctx.has_file(c) && !ctx.claudeignore_contains(c))
            .collect();

        if missing.is_empty() {
            self.pass()
        } else {
            self.fail(
                &format!("Build caches not in .claudeignore: {}", missing.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(", ")),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Add build caches to .claudeignore".into(),
                    description: format!("Add: {}", missing.iter().map(|m| format!("{m}/")).collect::<Vec<_>>().join(", ")),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule FE2: Source maps ignored ───────────────────────────────────

pub struct SourceMapsIgnored;

impl Rule for SourceMapsIgnored {
    fn id(&self) -> &str { "FE2" }
    fn name(&self) -> &str { "Source maps are ignored" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_frontend(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let has_maps = ctx.all_files.iter().any(|f| {
            f.path.extension().map(|e| e == "map").unwrap_or(false)
        });

        if !has_maps {
            return self.pass();
        }

        if ctx.claudeignore_contains(".map") || ctx.claudeignore_contains("*.map") {
            self.pass()
        } else {
            self.fail(
                "Source map files (.map) found but not in .claudeignore",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Ignore source maps".into(),
                    description: "Add *.map to .claudeignore. Source maps can be larger than the original source and Claude never needs them.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule FE3: .env.example companion ────────────────────────────────

pub struct EnvExampleExists;

impl Rule for EnvExampleExists {
    fn id(&self) -> &str { "FE3" }
    fn name(&self) -> &str { ".env.example documents env vars" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_frontend(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let has_real_env = ctx.env_files.iter().any(|f| {
            let name = f.file_name().and_then(|n| n.to_str()).unwrap_or("");
            !name.contains("example") && !name.contains("sample")
        });

        if !has_real_env {
            return self.pass();
        }

        let has_example = ctx.has_file(".env.example") || ctx.has_file(".env.sample");
        if has_example {
            self.pass()
        } else {
            self.fail(
                ".env file exists but no .env.example to document required variables",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Create .env.example".into(),
                    description: "Create .env.example with all required variable names (no real values). Claude reads this to understand the env shape without seeing secrets.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule FE4: Type-check before build ───────────────────────────────

pub struct TypeCheckBeforeBuild;

impl Rule for TypeCheckBeforeBuild {
    fn id(&self) -> &str { "FE4" }
    fn name(&self) -> &str { "Type-check prioritized over build" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_frontend(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.skip(),
        };

        let has_typecheck = content.contains("tsc")
            || content.contains("type-check")
            || content.contains("typecheck")
            || content.contains("--noemit");

        if has_typecheck {
            self.pass()
        } else {
            self.warn(
                "CLAUDE.md doesn't mention type-checking (tsc --noEmit)",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Add type-check command to CLAUDE.md".into(),
                    description: "Add `tsc --noEmit` or equivalent. Type-checking catches 90% of errors in 5 seconds vs 2+ minutes for a full build.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}
