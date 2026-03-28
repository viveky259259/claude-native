use crate::detection::{PrimaryType, ProjectType};
use crate::rules::*;
use crate::scan::ProjectContext;

fn is_serverless(pt: &ProjectType) -> bool {
    matches!(pt.primary, PrimaryType::Serverless(_))
}

// ── Rule SLS1: Deployment artifacts ignored ─────────────────────────

pub struct DeployArtifactsIgnored;

impl Rule for DeployArtifactsIgnored {
    fn id(&self) -> &str { "SLS1" }
    fn name(&self) -> &str { "Deployment artifacts are ignored" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_serverless(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let artifacts = [".aws-sam", ".serverless", ".vercel", "cdk.out"];
        let present: Vec<&&str> = artifacts.iter().filter(|a| ctx.root.join(a).is_dir()).collect();

        if present.is_empty() {
            return self.pass();
        }

        let all_ignored = present.iter().all(|a| ctx.claudeignore_contains(a));
        if all_ignored {
            self.pass()
        } else {
            self.fail(
                &format!("Deployment artifacts not in .claudeignore: {}", present.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(", ")),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Ignore deployment artifacts".into(),
                    description: format!("Add to .claudeignore: {}. sam build generates 10,000+ line CloudFormation templates.", present.iter().map(|a| format!("{a}/")).collect::<Vec<_>>().join(", ")),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule SLS2: Functions have focused scope ─────────────────────────

pub struct FunctionsFocused;

impl Rule for FunctionsFocused {
    fn id(&self) -> &str { "SLS2" }
    fn name(&self) -> &str { "Function handlers are <100 lines" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_serverless(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let handler_patterns = ["handler", "index", "lambda", "function"];
        let large_handlers: Vec<_> = ctx.all_files.iter()
            .filter(|f| {
                let name = f.path.file_stem().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
                handler_patterns.iter().any(|p| name.contains(p)) && f.line_count > 100
            })
            .collect();

        if large_handlers.is_empty() {
            self.pass()
        } else {
            let examples: Vec<String> = large_handlers.iter().take(3)
                .map(|f| format!("{} ({} lines)", f.relative_path.display(), f.line_count))
                .collect();
            self.warn(
                &format!("Large handler files: {}", examples.join(", ")),
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Keep handlers under 100 lines".into(),
                    description: "Serverless functions should be small by design. Extract business logic into separate modules and keep handlers thin.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule SLS3: Test events exist ────────────────────────────────────

pub struct TestEventsExist;

impl Rule for TestEventsExist {
    fn id(&self) -> &str { "SLS3" }
    fn name(&self) -> &str { "Test event files exist" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_serverless(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let has_events = ctx.root.join("events").is_dir()
            || ctx.root.join("test-events").is_dir()
            || ctx.all_files.iter().any(|f| {
                f.relative_path.to_string_lossy().contains("event") && f.path.extension().map(|e| e == "json").unwrap_or(false)
            });

        if has_events {
            self.pass()
        } else {
            self.warn(
                "No test event files found (events/ or test-events/)",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Add test event files".into(),
                    description: "Create events/ directory with sample event JSON files. Claude needs these to test functions locally with `sam local invoke`.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule SLS4: Cloud credentials not in repo ────────────────────────

pub struct NoCloudCreds;

impl Rule for NoCloudCreds {
    fn id(&self) -> &str { "SLS4" }
    fn name(&self) -> &str { "Cloud credentials not in repo" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Critical }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_serverless(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let cred_dirs = [".aws", "credentials"];
        let found: Vec<&&str> = cred_dirs.iter().filter(|d| ctx.root.join(d).exists()).collect();

        if found.is_empty() {
            self.pass()
        } else {
            self.fail(
                &format!("Cloud credential files/directories found: {}", found.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", ")),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Remove cloud credentials from repo".into(),
                    description: "Serverless projects have direct cloud access. Leaked credentials = full cloud compromise. Add to .gitignore immediately.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}
