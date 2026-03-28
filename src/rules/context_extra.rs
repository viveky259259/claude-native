use crate::rules::*;
use crate::scan::ProjectContext;

// ── Rule 2.5: No secrets in the repo ────────────────────────────────

pub struct NoSecretsInRepo;

impl Rule for NoSecretsInRepo {
    fn id(&self) -> &str { "2.5" }
    fn name(&self) -> &str { "No secrets in the repository" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Critical }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.env_files.is_empty() { return self.pass(); }
        let real: Vec<_> = ctx.env_files.iter()
            .filter(|f| {
                let name = f.file_name().and_then(|n| n.to_str()).unwrap_or("");
                !name.contains("example") && !name.contains("sample") && !name.contains("template")
            })
            .collect();
        if real.is_empty() { return self.pass(); }
        let names: Vec<String> = real.iter().take(5)
            .filter_map(|f| f.file_name().and_then(|n| n.to_str()).map(String::from))
            .collect();
        self.fail(
            &format!("Potential secret files: {}", names.join(", ")),
            Suggestion {
                priority: SuggestionPriority::QuickWin,
                title: "Remove/ignore secret files".into(),
                description: "Add .env, .env.*, *.pem, *.key to .gitignore AND .claudeignore.".into(),
                effort: Effort::Minutes,
            },
        )
    }
}

// ── Rule 2.6: README exists and is concise ──────────────────────────

pub struct ReadmeExistsAndConcise;

impl Rule for ReadmeExistsAndConcise {
    fn id(&self) -> &str { "2.6" }
    fn name(&self) -> &str { "README exists and is concise" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.readme_content.is_none() {
            return self.fail("No README.md found", Suggestion {
                priority: SuggestionPriority::NiceToHave,
                title: "Create README.md".into(),
                description: "Create a concise README.md (<300 lines).".into(),
                effort: Effort::Hour,
            });
        }
        let lines = ctx.readme_line_count();
        if lines <= 300 { self.pass() }
        else {
            self.warn(
                &format!("README.md is {lines} lines (target: <300)"),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Trim README.md".into(),
                    description: "Move detailed docs to docs/.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule 2.7: Subdirectory CLAUDE.md for large projects ─────────────

pub struct SubdirClaudeMd;

impl Rule for SubdirClaudeMd {
    fn id(&self) -> &str { "2.7" }
    fn name(&self) -> &str { "Subdirectory CLAUDE.md for large projects" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let count = ctx.source_file_count();
        if count <= 20 { return self.pass(); }
        if !ctx.subdirectory_claude_mds.is_empty() { self.pass() }
        else {
            self.warn(
                &format!("{count} source files but no subdirectory CLAUDE.md files"),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Add CLAUDE.md to major subdirectories".into(),
                    description: "Subdirectory CLAUDE.md files load on-demand, saving tokens.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}
