use crate::detection::{PrimaryType, ProjectType};
use crate::rules::*;
use crate::scan::ProjectContext;

// ── Rule μ1: README is the primary documentation ────────────────────

pub struct ReadmeIsPrimary;

impl Rule for ReadmeIsPrimary {
    fn id(&self) -> &str { "μ1" }
    fn name(&self) -> &str { "README is primary documentation" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool {
        matches!(pt.primary, PrimaryType::MicroRepo)
    }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.readme_content.is_none() {
            return self.fail(
                "Micro-repo has no README.md — this is the primary documentation for consumers",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Create README.md".into(),
                    description: "For micro-repos, README is THE documentation. Include: purpose, installation, usage examples, API surface. Claude reads this to understand the package contract.".into(),
                    effort: Effort::Hour,
                },
            );
        }

        let _lines = ctx.readme_line_count();
        let content = ctx.readme_content.as_ref().unwrap();
        let lower = content.to_lowercase();

        let has_install = lower.contains("install") || lower.contains("setup") || lower.contains("getting started");
        let has_usage = lower.contains("usage") || lower.contains("example") || lower.contains("```");

        if has_install && has_usage {
            self.pass()
        } else {
            self.warn(
                "README.md may be missing installation or usage sections",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Add install/usage to README".into(),
                    description: "README should include installation and usage examples. Claude uses these to understand how consumers interact with your package.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule μ2: Comprehensive tests ────────────────────────────────────

pub struct ComprehensiveTests;

impl Rule for ComprehensiveTests {
    fn id(&self) -> &str { "μ2" }
    fn name(&self) -> &str { "Comprehensive tests (micro-repo)" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool {
        matches!(pt.primary, PrimaryType::MicroRepo)
    }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let source_count = ctx.source_file_count();
        let test_files = ctx.test_files.len();
        let test_fns = ctx.test_function_count();

        if source_count == 0 {
            return self.skip();
        }

        // Use the better of two metrics: file ratio OR function-to-source ratio
        let file_ratio = test_files as f64 / source_count as f64;
        let fn_ratio = test_fns as f64 / source_count as f64;
        let best_ratio = file_ratio.max(fn_ratio);

        if best_ratio >= 0.5 {
            self.pass()
        } else if best_ratio >= 0.2 {
            self.warn(
                &format!("{test_fns} test functions in {test_files} files for {source_count} source files ({:.0}%).", best_ratio * 100.0),
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Add more tests".into(),
                    description: "Micro-repos need higher coverage. Aim for at least 1 test per 2 source files.".into(),
                    effort: Effort::HalfDay,
                },
            )
        } else {
            self.fail(
                &format!("Micro-repo has {test_fns} test functions for {source_count} source files — needs more coverage"),
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Significantly improve test coverage".into(),
                    description: "Micro-repos need high test coverage. Claude can't verify changes without tests.".into(),
                    effort: Effort::HalfDay,
                },
            )
        }
    }
}

// ── Rule μ3: Package manifest is complete ───────────────────────────

pub struct ManifestComplete;

impl Rule for ManifestComplete {
    fn id(&self) -> &str { "μ3" }
    fn name(&self) -> &str { "Package manifest is complete" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool {
        matches!(pt.primary, PrimaryType::MicroRepo)
    }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        // Check package.json completeness
        if let Some(pj) = &ctx.package_json {
            let has_name = pj.get("name").is_some();
            let has_version = pj.get("version").is_some();
            let has_description = pj.get("description").and_then(|d| d.as_str()).map(|s| !s.is_empty()).unwrap_or(false);
            let has_license = pj.get("license").is_some();

            let mut missing = Vec::new();
            if !has_name { missing.push("name"); }
            if !has_version { missing.push("version"); }
            if !has_description { missing.push("description"); }
            if !has_license { missing.push("license"); }

            if missing.is_empty() {
                return self.pass();
            } else {
                return self.warn(
                    &format!("package.json missing: {}", missing.join(", ")),
                    Suggestion {
                        priority: SuggestionPriority::NiceToHave,
                        title: "Complete package.json fields".into(),
                        description: format!("Add missing fields: {}. Claude uses the manifest to understand the package contract and how consumers use it.", missing.join(", ")),
                        effort: Effort::Minutes,
                    },
                );
            }
        }

        // Check Cargo.toml completeness
        if let Some(content) = ctx.read_root_file("Cargo.toml") {
            let has_description = content.contains("description");
            let has_license = content.contains("license");

            if has_description && has_license {
                return self.pass();
            } else {
                let mut missing = Vec::new();
                if !has_description { missing.push("description"); }
                if !has_license { missing.push("license"); }
                return self.warn(
                    &format!("Cargo.toml missing: {}", missing.join(", ")),
                    Suggestion {
                        priority: SuggestionPriority::NiceToHave,
                        title: "Complete Cargo.toml fields".into(),
                        description: format!("Add missing fields: {}. These help Claude understand the package purpose.", missing.join(", ")),
                        effort: Effort::Minutes,
                    },
                );
            }
        }

        self.pass()
    }
}

// ── Rule μ4: Examples exist ─────────────────────────────────────────

pub struct ExamplesExist;

impl Rule for ExamplesExist {
    fn id(&self) -> &str { "μ4" }
    fn name(&self) -> &str { "Examples directory or inline examples" }
    fn dimension(&self) -> Dimension { Dimension::Navigation }
    fn severity(&self) -> Severity { Severity::Low }

    fn applies_to(&self, pt: &ProjectType) -> bool {
        matches!(pt.primary, PrimaryType::MicroRepo)
    }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let has_examples_dir = ctx.has_file("examples") || ctx.root.join("examples").is_dir();
        let has_readme_examples = ctx.readme_content.as_ref().map(|c| {
            c.contains("```") && (c.to_lowercase().contains("example") || c.to_lowercase().contains("usage"))
        }).unwrap_or(false);

        if has_examples_dir || has_readme_examples {
            self.pass()
        } else {
            self.warn(
                "No examples/ directory or code examples in README",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Add usage examples".into(),
                    description: "Create an examples/ directory or add code examples to README. Claude uses examples to understand intended usage patterns.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}
