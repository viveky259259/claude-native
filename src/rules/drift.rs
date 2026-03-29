use crate::rules::*;
use crate::scan::ProjectContext;

// ── Rule 6.1: CLAUDE.md build command matches manifest ──────────────

pub struct BuildCommandMatchesManifest;

impl Rule for BuildCommandMatchesManifest {
    fn id(&self) -> &str { "6.1" }
    fn name(&self) -> &str { "CLAUDE.md build cmd matches manifest" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.skip(),
        };

        // Check for mismatches between CLAUDE.md commands and actual manifests
        let has_cargo = ctx.has_file("Cargo.toml");
        let has_npm = ctx.has_file("package.json");
        let has_go = ctx.has_file("go.mod");
        let has_python = ctx.has_file("requirements.txt") || ctx.has_file("pyproject.toml");

        let mentions_cargo = content.contains("cargo ");
        let mentions_npm = content.contains("npm ") || content.contains("npx ");
        let mentions_go = content.contains("go build") || content.contains("go test");
        let mentions_python = content.contains("python ") || content.contains("pytest") || content.contains("pip ");

        let mismatches = check_mismatches(
            has_cargo, has_npm, has_go, has_python,
            mentions_cargo, mentions_npm, mentions_go, mentions_python,
        );

        if mismatches.is_empty() {
            self.pass()
        } else {
            self.warn(
                &format!("CLAUDE.md may be stale: {}", mismatches.join("; ")),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Update CLAUDE.md commands".into(),
                    description: format!("CLAUDE.md references commands that don't match your project: {}. Update to match actual tooling.", mismatches.join(", ")),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

fn check_mismatches(
    has_cargo: bool, has_npm: bool, has_go: bool, has_python: bool,
    mentions_cargo: bool, mentions_npm: bool, mentions_go: bool, mentions_python: bool,
) -> Vec<String> {
    let mut m = Vec::new();
    if mentions_cargo && !has_cargo { m.push("mentions cargo but no Cargo.toml".into()); }
    if mentions_npm && !has_npm { m.push("mentions npm but no package.json".into()); }
    if mentions_go && !has_go { m.push("mentions go but no go.mod".into()); }
    if mentions_python && !has_python { m.push("mentions python but no requirements.txt/pyproject.toml".into()); }
    m
}

// ── Rule 6.2: Referenced files exist ────────────────────────────────

pub struct ReferencedFilesExist;

impl Rule for ReferencedFilesExist {
    fn id(&self) -> &str { "6.2" }
    fn name(&self) -> &str { "Files referenced in CLAUDE.md exist" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c,
            None => return self.skip(),
        };

        let missing = find_missing_references(content, ctx);

        if missing.is_empty() {
            self.pass()
        } else {
            self.warn(
                &format!("CLAUDE.md references files that don't exist: {}", missing.join(", ")),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Fix stale file references in CLAUDE.md".into(),
                    description: format!("These paths in CLAUDE.md don't exist: {}. Either create them or update CLAUDE.md.", missing.join(", ")),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

fn find_missing_references(content: &str, ctx: &ProjectContext) -> Vec<String> {
    let mut missing = Vec::new();
    // Look for backtick-quoted paths that look like file references
    for segment in content.split('`') {
        let trimmed = segment.trim();
        if looks_like_file_path(trimmed) && !ctx.has_file(trimmed) {
            missing.push(trimmed.to_string());
        }
    }
    // Deduplicate
    missing.sort();
    missing.dedup();
    missing.truncate(5); // limit output
    missing
}

fn looks_like_file_path(s: &str) -> bool {
    // Must contain / or . extension, not be a command, and be reasonable length
    let is_path = (s.contains('/') || s.contains('.'))
        && s.len() > 3
        && s.len() < 100
        && !s.contains(' ')
        && !s.starts_with("http")
        && !s.starts_with("--")
        && !s.starts_with("npm ")
        && !s.starts_with("cargo ")
        && !s.starts_with("go ");
    // Must look like a relative file path
    is_path && (s.ends_with('/') || s.contains('.'))
        && !s.starts_with('$')
        && !s.contains('(')
}
