use crate::detection::{PrimaryType, ProjectType};
use crate::rules::*;
use crate::scan::ProjectContext;

fn is_backend(pt: &ProjectType) -> bool {
    matches!(pt.primary, PrimaryType::Backend(_))
}

// ── Rule BE1: Migration history is manageable ───────────────────────

pub struct MigrationHistoryManageable;

impl Rule for MigrationHistoryManageable {
    fn id(&self) -> &str { "BE1" }
    fn name(&self) -> &str { "Migration history is manageable" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_backend(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let migration_dirs = ["migrations", "db/migrate", "alembic/versions", "prisma/migrations"];
        let mut total_migrations = 0;

        for dir in &migration_dirs {
            let path = ctx.root.join(dir);
            if path.is_dir() {
                if let Ok(entries) = std::fs::read_dir(&path) {
                    total_migrations += entries.filter_map(|e| e.ok()).filter(|e| e.path().is_file()).count();
                }
            }
        }

        if total_migrations == 0 {
            return self.pass();
        }

        if total_migrations <= 100 {
            self.pass()
        } else if total_migrations <= 200 {
            self.warn(
                &format!("{total_migrations} migration files — consider squashing old migrations"),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Squash old migrations".into(),
                    description: "Large migration histories waste tokens when Claude reads them to understand schema. Squash migrations older than 6 months.".into(),
                    effort: Effort::HalfDay,
                },
            )
        } else {
            self.fail(
                &format!("{total_migrations} migration files — too many for Claude to navigate efficiently"),
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Squash migration history".into(),
                    description: format!("{total_migrations} migrations waste thousands of tokens. Squash to <100 and document the current schema in CLAUDE.md."),
                    effort: Effort::HalfDay,
                },
            )
        }
    }
}

// ── Rule BE2: Database files ignored ────────────────────────────────

pub struct DatabaseFilesIgnored;

impl Rule for DatabaseFilesIgnored {
    fn id(&self) -> &str { "BE2" }
    fn name(&self) -> &str { "Database state files are ignored" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_backend(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let db_files: Vec<&str> = ["db.sqlite3", "database.sqlite", "dev.db"]
            .iter()
            .filter(|f| ctx.has_file(f))
            .copied()
            .collect();

        if db_files.is_empty() {
            return self.pass();
        }

        let ignored = ctx.claudeignore_contains("sqlite") || ctx.claudeignore_contains(".db");
        if ignored {
            self.pass()
        } else {
            self.fail(
                &format!("Database files found but not ignored: {}", db_files.join(", ")),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Ignore database files".into(),
                    description: "Add *.sqlite3, *.db to .claudeignore AND .gitignore. Database files can be 100MB+ and Claude reads schema from migrations/models, not data files.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule BE3: Virtual environments ignored ──────────────────────────

pub struct VirtualEnvsIgnored;

impl Rule for VirtualEnvsIgnored {
    fn id(&self) -> &str { "BE3" }
    fn name(&self) -> &str { "Virtual environments are ignored" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Critical }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_backend(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let venvs = [".venv", "venv", "vendor", "__pycache__"];
        let present: Vec<&&str> = venvs.iter().filter(|v| ctx.root.join(v).is_dir()).collect();

        if present.is_empty() {
            return self.pass();
        }

        let all_ignored = present.iter().all(|v| ctx.claudeignore_contains(v));
        if all_ignored {
            self.pass()
        } else {
            self.fail(
                &format!("Virtual environments present but not in .claudeignore: {}", present.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Ignore virtual environments".into(),
                    description: "Add to .claudeignore: .venv/, venv/, vendor/, __pycache__/. These can be 100MB+ and provide zero context.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule BE4: ORM/data access pattern documented ────────────────────

pub struct DataAccessDocumented;

impl Rule for DataAccessDocumented {
    fn id(&self) -> &str { "BE4" }
    fn name(&self) -> &str { "ORM/data access pattern documented" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_backend(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.skip(),
        };

        let has_data_docs = content.contains("orm")
            || content.contains("database")
            || content.contains("query")
            || content.contains("model")
            || content.contains("repository")
            || content.contains("prisma")
            || content.contains("sqlalchemy")
            || content.contains("active record");

        if has_data_docs {
            self.pass()
        } else {
            self.warn(
                "CLAUDE.md doesn't document the data access pattern",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Document data access pattern".into(),
                    description: "Add to CLAUDE.md which ORM/query pattern to use. Backend projects often have 3+ ways to query data — Claude picks wrong 33% of the time without guidance.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule BE5: API spec exists ───────────────────────────────────────

pub struct ApiSpecExists;

impl Rule for ApiSpecExists {
    fn id(&self) -> &str { "BE5" }
    fn name(&self) -> &str { "API spec exists (OpenAPI/Swagger)" }
    fn dimension(&self) -> Dimension { Dimension::Navigation }
    fn severity(&self) -> Severity { Severity::Low }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_backend(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let has_spec = ctx.has_file("openapi.yaml")
            || ctx.has_file("openapi.yml")
            || ctx.has_file("openapi.json")
            || ctx.has_file("swagger.yaml")
            || ctx.has_file("swagger.yml")
            || ctx.has_file("swagger.json")
            || ctx.has_file("api-spec.yaml");

        if has_spec {
            self.pass()
        } else {
            self.warn(
                "No OpenAPI/Swagger API spec found",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Add API specification".into(),
                    description: "Create openapi.yaml or swagger.json. Claude uses API specs to understand endpoints and request/response shapes without reading every handler.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule BE6: Log files ignored ─────────────────────────────────────

pub struct LogFilesIgnored;

impl Rule for LogFilesIgnored {
    fn id(&self) -> &str { "BE6" }
    fn name(&self) -> &str { "Log files are ignored" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_backend(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let has_logs = ctx.root.join("logs").is_dir()
            || ctx.root.join("log").is_dir()
            || ctx.all_files.iter().any(|f| f.path.extension().map(|e| e == "log").unwrap_or(false));

        if !has_logs {
            return self.pass();
        }

        let ignored = ctx.claudeignore_contains("*.log")
            || ctx.claudeignore_contains("logs/")
            || ctx.claudeignore_contains("log/");

        if ignored {
            self.pass()
        } else {
            self.fail(
                "Log files/directories found but not in .claudeignore",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Ignore log files".into(),
                    description: "Add *.log, logs/, tmp/ to .claudeignore. A single log file can be 100MB — Claude should use filtered commands instead.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}
