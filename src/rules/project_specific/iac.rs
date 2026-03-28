use crate::detection::{PrimaryType, ProjectType};
use crate::rules::*;
use crate::scan::ProjectContext;

fn is_iac(pt: &ProjectType) -> bool {
    matches!(pt.primary, PrimaryType::IaC(_))
}

// ── Rule IAC1: State files are NEVER readable ───────────────────────

pub struct StateFilesBlocked;

impl Rule for StateFilesBlocked {
    fn id(&self) -> &str { "IAC1" }
    fn name(&self) -> &str { "State files are blocked from reading" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Critical }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_iac(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let state_files = ["terraform.tfstate", "terraform.tfstate.backup"];
        let has_state = state_files.iter().any(|f| ctx.has_file(f));

        if !has_state {
            // Check .terraform dir
            if ctx.root.join(".terraform").is_dir()
                && !ctx.claudeignore_contains(".terraform")
            {
                return self.fail(
                    ".terraform/ directory not in .claudeignore — contains provider cache and may contain state",
                    Suggestion {
                        priority: SuggestionPriority::QuickWin,
                        title: "Ignore .terraform/ directory".into(),
                        description: "Add .terraform/ to .claudeignore. It contains provider plugins (large binaries) and possibly state with secrets.".into(),
                        effort: Effort::Minutes,
                    },
                );
            }
            return self.pass();
        }

        let ignored = ctx.claudeignore_contains("tfstate");
        if ignored {
            self.pass()
        } else {
            self.fail(
                "Terraform state files exist but are not blocked in .claudeignore — SECURITY RISK: state contains secrets",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Block state files immediately".into(),
                    description: "Add *.tfstate, *.tfstate.backup, .terraform/ to .claudeignore AND .gitignore. State files contain ALL resource attributes including passwords, tokens, and private keys.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule IAC2: Plan output is filtered ──────────────────────────────

pub struct PlanOutputFiltered;

impl Rule for PlanOutputFiltered {
    fn id(&self) -> &str { "IAC2" }
    fn name(&self) -> &str { "Plan output filtering documented" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_iac(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.skip(),
        };

        let mentions_filtering = content.contains("filter")
            || content.contains("grep")
            || content.contains("head")
            || content.contains("show")
            || content.contains("plan");

        if mentions_filtering {
            self.pass()
        } else {
            self.warn(
                "CLAUDE.md doesn't mention filtering plan output",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Document plan output filtering".into(),
                    description: "Add to CLAUDE.md: 'For terraform plan, filter to resource changes only. Raw plan output can be 50,000 lines.' Suggest using `terraform show tfplan | head -200`.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule IAC3: Provider cache ignored ───────────────────────────────

pub struct ProviderCacheIgnored;

impl Rule for ProviderCacheIgnored {
    fn id(&self) -> &str { "IAC3" }
    fn name(&self) -> &str { "Provider/plugin cache is ignored" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_iac(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let caches = [".terraform", ".pulumi", "cdk.out", "cdktf.out"];
        let present: Vec<&&str> = caches.iter().filter(|c| ctx.root.join(c).is_dir()).collect();

        if present.is_empty() {
            return self.pass();
        }

        let all_ignored = present.iter().all(|c| ctx.claudeignore_contains(c));
        if all_ignored {
            self.pass()
        } else {
            self.fail(
                &format!("IaC cache dirs not in .claudeignore: {}", present.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(", ")),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Ignore IaC cache directories".into(),
                    description: format!("Add to .claudeignore: {}", present.iter().map(|c| format!("{c}/")).collect::<Vec<_>>().join(", ")),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule IAC4: Secrets managed externally ───────────────────────────

pub struct SecretsExternal;

impl Rule for SecretsExternal {
    fn id(&self) -> &str { "IAC4" }
    fn name(&self) -> &str { "Secrets managed externally" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Critical }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_iac(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        // Check for terraform.tfvars with real values
        if let Ok(content) = std::fs::read_to_string(ctx.root.join("terraform.tfvars")) {
            let has_real_values = content.lines().any(|l| {
                let l = l.trim();
                !l.is_empty() && !l.starts_with('#') && l.contains('=')
                    && !l.contains("var.") && !l.contains("${")
            });
            if has_real_values {
                return self.fail(
                    "terraform.tfvars may contain real secret values",
                    Suggestion {
                        priority: SuggestionPriority::QuickWin,
                        title: "Externalize secrets from tfvars".into(),
                        description: "Move secrets to environment variables, HashiCorp Vault, or AWS SSM. IaC is the highest risk for credential exposure. Use terraform.tfvars.example instead.".into(),
                        effort: Effort::Hour,
                    },
                );
            }
        }
        self.pass()
    }
}

// ── Rule IAC5: Module structure follows conventions ──────────────────

pub struct ModuleConventions;

impl Rule for ModuleConventions {
    fn id(&self) -> &str { "IAC5" }
    fn name(&self) -> &str { "Module structure uses conventions" }
    fn dimension(&self) -> Dimension { Dimension::Navigation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_iac(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let has_variables = ctx.has_file("variables.tf");
        let has_outputs = ctx.has_file("outputs.tf");
        let has_main = ctx.has_file("main.tf");

        if has_variables && has_outputs && has_main {
            self.pass()
        } else {
            let mut missing = Vec::new();
            if !has_variables { missing.push("variables.tf"); }
            if !has_outputs { missing.push("outputs.tf"); }
            if !has_main { missing.push("main.tf"); }
            self.warn(
                &format!("Missing standard Terraform files: {}", missing.join(", ")),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Follow Terraform module conventions".into(),
                    description: format!("Create: {}. Claude navigates modules by reading variables.tf first (contract), then main.tf (implementation).", missing.join(", ")),
                    effort: Effort::Hour,
                },
            )
        }
    }
}
