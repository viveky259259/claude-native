use crate::detection::ProjectType;
use crate::rules::*;
use crate::scan::ProjectContext;

fn is_polyglot(pt: &ProjectType) -> bool {
    pt.flags.is_polyglot
}

// ── Rule POLY1: Each language has its own CLAUDE.md ─────────────────

pub struct PerLanguageClaudeMd;

impl Rule for PerLanguageClaudeMd {
    fn id(&self) -> &str { "POLY1" }
    fn name(&self) -> &str { "Per-language CLAUDE.md files" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_polyglot(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let lang_dirs = ["backend", "frontend", "server", "client", "api", "web", "app", "service"];
        let dirs_with_claude_md = lang_dirs.iter()
            .filter(|d| {
                let dir = ctx.root.join(d);
                dir.is_dir() && (dir.join("CLAUDE.md").exists() || dir.join(".claude").join("CLAUDE.md").exists())
            })
            .count();

        let relevant_dirs = lang_dirs.iter().filter(|d| ctx.root.join(d).is_dir()).count();

        if relevant_dirs == 0 {
            return self.pass();
        }

        if dirs_with_claude_md >= relevant_dirs / 2 {
            self.pass()
        } else {
            self.fail(
                &format!("Only {dirs_with_claude_md}/{relevant_dirs} language directories have CLAUDE.md"),
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Add CLAUDE.md per language directory".into(),
                    description: "Each language directory needs its own CLAUDE.md with language-specific conventions. Go conventions waste tokens when Claude works in TypeScript.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule POLY2: Root CLAUDE.md is language-agnostic ─────────────────

pub struct RootClaudeMdAgnostic;

impl Rule for RootClaudeMdAgnostic {
    fn id(&self) -> &str { "POLY2" }
    fn name(&self) -> &str { "Root CLAUDE.md is language-agnostic" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_polyglot(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c,
            None => return self.skip(),
        };

        // Check if root CLAUDE.md contains language-specific details that should be in subdirs
        let lower = content.to_lowercase();
        let lang_specific_patterns = [
            "import React", "from django", "func main()", "fn main()",
            "package.json", "Cargo.toml", "go.mod", "requirements.txt",
        ];

        let lang_refs: Vec<&&str> = lang_specific_patterns.iter()
            .filter(|p| content.contains(*p) || lower.contains(&p.to_lowercase()))
            .collect();

        if lang_refs.len() > 2 {
            self.warn(
                "Root CLAUDE.md contains language-specific details that should be in subdirectory CLAUDE.md files",
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Move language rules to subdirectories".into(),
                    description: "Root CLAUDE.md loads on EVERY request. Language-specific rules (50%+ of the time irrelevant) should live in backend/CLAUDE.md, frontend/CLAUDE.md, etc.".into(),
                    effort: Effort::Hour,
                },
            )
        } else {
            self.pass()
        }
    }
}

// ── Rule POLY3: Independent build/test per language ─────────────────

pub struct IndependentBuildTest;

impl Rule for IndependentBuildTest {
    fn id(&self) -> &str { "POLY3" }
    fn name(&self) -> &str { "Independent build/test per language" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_polyglot(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.skip(),
        };

        // Check for per-language commands
        let has_multiple_cmds = (content.contains("cd ") && content.contains("test"))
            || (content.matches("test").count() >= 2)
            || content.contains("backend") && content.contains("frontend");

        if has_multiple_cmds || !ctx.subdirectory_claude_mds.is_empty() {
            self.pass()
        } else {
            self.fail(
                "No per-language build/test commands found",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Document per-language commands".into(),
                    description: "Add separate build/test commands per language: `cd backend && go test` and `cd frontend && npm test`. A single `make test` that runs everything takes too long.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule POLY4: .claudeignore covers ALL runtimes ───────────────────

pub struct AllRuntimesIgnored;

impl Rule for AllRuntimesIgnored {
    fn id(&self) -> &str { "POLY4" }
    fn name(&self) -> &str { ".claudeignore covers all language runtimes" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_polyglot(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.claudeignore_content.is_none() {
            return self.fail(
                "No .claudeignore in polyglot project — ALL language runtime dirs are visible",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Create .claudeignore for all languages".into(),
                    description: "A polyglot project needs: node_modules/, vendor/, .venv/, target/, __pycache__/, .gradle/ — miss one and half the noise leaks through.".into(),
                    effort: Effort::Minutes,
                },
            );
        }

        let runtime_dirs = [
            ("node_modules", "JavaScript/TypeScript"),
            ("vendor", "Go/PHP"),
            (".venv", "Python"),
            ("venv", "Python"),
            ("target", "Rust"),
            ("__pycache__", "Python"),
            (".gradle", "Kotlin/Java"),
        ];

        let missing: Vec<(&str, &str)> = runtime_dirs.iter()
            .filter(|(dir, _)| ctx.root.join(dir).is_dir() && !ctx.claudeignore_contains(dir))
            .copied()
            .collect();

        if missing.is_empty() {
            self.pass()
        } else {
            let list: Vec<String> = missing.iter().map(|(d, l)| format!("{d}/ ({l})")).collect();
            self.fail(
                &format!("Runtime dirs not ignored: {}", list.join(", ")),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Ignore all runtime directories".into(),
                    description: format!("Add to .claudeignore: {}", missing.iter().map(|(d, _)| format!("{d}/")).collect::<Vec<_>>().join(", ")),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}
