use crate::rules::*;
use crate::scan::ProjectContext;

// ── Rule 5.3: Descriptive test names ────────────────────────────────

pub struct DescriptiveTestNames;

impl Rule for DescriptiveTestNames {
    fn id(&self) -> &str { "5.3" }
    fn name(&self) -> &str { "Tests have descriptive names" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.test_files.is_empty() {
            return self.skip();
        }
        let (bad_names, total_tests) = scan_test_names(ctx);
        if total_tests == 0 || bad_names == 0 {
            self.pass()
        } else {
            self.warn(
                &format!("{bad_names}/{total_tests} test names are non-descriptive"),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Use descriptive test names".into(),
                    description: "Replace 'test1' with behavior: 'test_login_rejects_invalid_email'.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

fn scan_test_names(ctx: &ProjectContext) -> (usize, usize) {
    let bad_patterns = ["test1", "test2", "test3", "test_1", "test_2"];
    let mut bad_names = 0;
    let mut total_tests = 0;

    for tf in &ctx.test_files {
        let content = match std::fs::read_to_string(tf) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for line in content.lines() {
            if !is_test_declaration(line.trim()) { continue; }
            total_tests += 1;
            let lower = line.trim().to_lowercase();
            if bad_patterns.iter().any(|p| lower.contains(p)) {
                bad_names += 1;
            }
        }
    }
    (bad_names, total_tests)
}

fn is_test_declaration(trimmed: &str) -> bool {
    trimmed.starts_with("fn test")
        || trimmed.starts_with("#[test]")
        || trimmed.starts_with("test(")
        || trimmed.starts_with("test '")
        || trimmed.starts_with("test \"")
        || trimmed.starts_with("it(")
        || trimmed.starts_with("def test_")
}

// ── Rule 5.4: Consistent patterns ──────────────────────────────────

pub struct ConsistentPatterns;

impl Rule for ConsistentPatterns {
    fn id(&self) -> &str { "5.4" }
    fn name(&self) -> &str { "Consistent patterns across codebase" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if let Some(content) = &ctx.claude_md_content {
            let l = content.to_lowercase();
            if l.contains("pattern") || l.contains("convention") || l.contains("style") {
                return self.pass();
            }
        }
        if ctx.source_file_count() > 10 {
            self.warn(
                "No documented code patterns found in CLAUDE.md",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Document code patterns in CLAUDE.md".into(),
                    description: "Describe error handling, data access, and API patterns.".into(),
                    effort: Effort::Minutes,
                },
            )
        } else {
            self.pass()
        }
    }
}

// ── Rule 5.5: Comments explain why ──────────────────────────────────

pub struct CommentsExplainWhy;

const WHAT_PATTERNS: &[&str] = &[
    "// set ", "// get ", "// return ", "// loop ", "// iterate ",
    "// initialize ", "// create ", "// assign ", "// increment ",
    "# set ", "# get ", "# return ", "# loop ", "# iterate ",
    "# initialize ", "# create ", "# assign ",
];

impl Rule for CommentsExplainWhy {
    fn id(&self) -> &str { "5.5" }
    fn name(&self) -> &str { "Comments explain 'why', not 'what'" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let (what_count, total) = count_what_comments(ctx);
        if total < 5 { return self.pass(); }
        let ratio = what_count as f64 / total as f64;
        if ratio > 0.3 {
            self.warn(
                &format!("{what_count}/{total} comments restate the code"),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Improve comment quality".into(),
                    description: "Replace 'what' comments with 'why' comments.".into(),
                    effort: Effort::Hour,
                },
            )
        } else {
            self.pass()
        }
    }
}

fn count_what_comments(ctx: &ProjectContext) -> (usize, usize) {
    let mut what_count = 0;
    let mut total = 0;
    for f in ctx.all_files.iter().filter(|f| !f.is_test && !f.is_generated).take(20) {
        let content = match std::fs::read_to_string(&f.path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for line in content.lines() {
            let t = line.trim().to_lowercase();
            if t.starts_with("//") || (t.starts_with('#') && !t.starts_with("#[")) {
                total += 1;
                if WHAT_PATTERNS.iter().any(|p| t.starts_with(p)) { what_count += 1; }
            }
        }
    }
    (what_count, total)
}

// ── Rule 5.6: No dead code ─────────────────────────────────────────

pub struct NoDeadCode;

impl Rule for NoDeadCode {
    fn id(&self) -> &str { "5.6" }
    fn name(&self) -> &str { "No dead code" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let dead = ["deprecated", "old", "backup", "archive", "legacy"];
        let found: Vec<String> = ctx.directories.iter()
            .filter_map(|d| {
                let name = d.file_name()?.to_str()?.to_lowercase();
                dead.contains(&name.as_str()).then_some(name)
            })
            .collect();
        if found.is_empty() { self.pass() }
        else {
            self.warn(
                &format!("Dead code directories: {}", found.join(", ")),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Remove or ignore dead code".into(),
                    description: "Add to .claudeignore so Claude skips deprecated code.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule 5.7: Dependencies documented ───────────────────────────────

pub struct DependenciesDocumented;

impl Rule for DependenciesDocumented {
    fn id(&self) -> &str { "5.7" }
    fn name(&self) -> &str { "Dependencies are documented" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if !ctx.package_manifests.is_empty() { self.pass() }
        else {
            self.fail(
                "No package manifest found",
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Add a package manifest".into(),
                    description: "Create package.json/Cargo.toml/requirements.txt.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule 5.8: CI/CD exists ─────────────────────────────────────────

pub struct CiCdExists;

impl Rule for CiCdExists {
    fn id(&self) -> &str { "5.8" }
    fn name(&self) -> &str { "CI/CD pipeline exists" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let has_ci = !ctx.ci_configs.is_empty()
            || ctx.root.join(".github").join("workflows").is_dir()
            || ctx.has_file(".gitlab-ci.yml")
            || ctx.has_file("Jenkinsfile");
        if has_ci { self.pass() }
        else {
            self.warn(
                "No CI/CD configuration found",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Add CI/CD pipeline".into(),
                    description: "Add .github/workflows/ or equivalent.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}
