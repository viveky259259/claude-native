use crate::detection::Language;
use crate::rules::*;
use crate::scan::ProjectContext;

// ── Rule 5.1: Type annotations exist ────────────────────────────────

pub struct TypeAnnotationsExist;

impl Rule for TypeAnnotationsExist {
    fn id(&self) -> &str { "5.1" }
    fn name(&self) -> &str { "Type annotations exist" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let pt = match &ctx.project_type {
            Some(pt) => pt,
            None => return self.skip(),
        };

        // Only relevant for dynamically-typed languages
        let needs_types = pt.languages.iter().any(|l| matches!(l,
            Language::JavaScript | Language::Python | Language::Ruby
        ));

        if !needs_types {
            return self.pass(); // Rust, Go, TypeScript, etc. have built-in types
        }

        // Check for TypeScript (upgrades JS)
        let has_typescript = pt.languages.iter().any(|l| matches!(l, Language::TypeScript));
        if has_typescript {
            return self.pass();
        }

        // Check for Python type hints (mypy, pyproject.toml with mypy config)
        if pt.languages.iter().any(|l| matches!(l, Language::Python)) {
            let has_mypy = ctx.has_file("mypy.ini")
                || ctx.has_file(".mypy.ini")
                || ctx.has_file("setup.cfg")
                || ctx.read_root_file("pyproject.toml")
                    .map(|c| c.contains("[tool.mypy]") || c.contains("mypy"))
                    .unwrap_or(false);
            if has_mypy {
                return self.pass();
            }
        }

        self.fail(
            "Dynamically-typed language detected without type annotations/checking",
            Suggestion {
                priority: SuggestionPriority::HighImpact,
                title: "Add type annotations".into(),
                description: "For JS: migrate to TypeScript or add JSDoc types. For Python: add type hints and mypy. Types give Claude contracts to work with — reducing hallucinated return values.".into(),
                effort: Effort::HalfDay,
            },
        )
    }
}

// ── Rule 5.2: Tests exist ──────────────────────────────────────────

pub struct TestsExist;

impl Rule for TestsExist {
    fn id(&self) -> &str { "5.2" }
    fn name(&self) -> &str { "Tests exist" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::High }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if !ctx.test_files.is_empty() {
            let ratio = ctx.test_files.len() as f64 / ctx.source_file_count().max(1) as f64;
            if ratio < 0.1 {
                return self.warn(
                    &format!("Only {} test files for {} source files ({:.0}% ratio)", ctx.test_files.len(), ctx.source_file_count(), ratio * 100.0),
                    Suggestion {
                        priority: SuggestionPriority::HighImpact,
                        title: "Add more tests".into(),
                        description: "Test coverage is very low. Tests are Claude's primary way to verify changes. Aim for at least 1 test file per 3 source files.".into(),
                        effort: Effort::HalfDay,
                    },
                );
            }
            self.pass()
        } else {
            self.fail(
                "No test files found",
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Add tests".into(),
                    description: "Tests are Claude's primary verification mechanism. Without them, Claude can't validate its own changes. Start with tests for critical paths.".into(),
                    effort: Effort::HalfDay,
                },
            )
        }
    }
}

// Rules 5.3-5.8 are in quality_extra.rs
