use crate::detection::{PrimaryType, ProjectType};
use crate::rules::*;
use crate::scan::ProjectContext;

fn is_codegen(pt: &ProjectType) -> bool {
    matches!(pt.primary, PrimaryType::CodegenHeavy)
}

// ── Rule GEN1: Generated code directories ignored ───────────────────

pub struct GeneratedDirsIgnored;

impl Rule for GeneratedDirsIgnored {
    fn id(&self) -> &str { "GEN1" }
    fn name(&self) -> &str { "Generated code directories are ignored" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Critical }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_codegen(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let gen_count = ctx.all_files.iter().filter(|f| f.is_generated).count();
        if gen_count == 0 {
            return self.pass();
        }

        let ignored = ctx.claudeignore_contains("generated")
            || ctx.claudeignore_contains("gen/")
            || ctx.claudeignore_contains("*.pb.")
            || ctx.claudeignore_contains("*.g.dart");

        if ignored {
            self.pass()
        } else {
            self.fail(
                &format!("{gen_count} generated files found but not excluded in .claudeignore"),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Ignore generated code directories".into(),
                    description: "Add to .claudeignore: src/generated/, gen/, *_generated.*, *.pb.go, *.pb.ts, *.g.dart. Claude should read specs, not generated output.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule GEN2: Edit specs, not generated code ───────────────────────

pub struct EditSpecsNotGenerated;

impl Rule for EditSpecsNotGenerated {
    fn id(&self) -> &str { "GEN2" }
    fn name(&self) -> &str { "CLAUDE.md says edit specs, not generated" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_codegen(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.skip(),
        };

        let has_instruction = content.contains("generated")
            || content.contains("don't edit")
            || content.contains("do not edit")
            || content.contains("auto-generated")
            || content.contains("codegen")
            || (content.contains("proto") && content.contains("edit"));

        if has_instruction {
            self.pass()
        } else {
            self.fail(
                "CLAUDE.md doesn't instruct to edit specs instead of generated code",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Add 'edit specs, not generated code' rule".into(),
                    description: "Add to CLAUDE.md: 'Edit .proto/.graphql/schema.prisma files — NEVER edit generated code. Regeneration overwrites changes.' This is the #1 mistake.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule GEN3: Regeneration command documented ──────────────────────

pub struct RegenCommandDocumented;

impl Rule for RegenCommandDocumented {
    fn id(&self) -> &str { "GEN3" }
    fn name(&self) -> &str { "Regeneration command is documented" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_codegen(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.skip(),
        };

        let has_regen = content.contains("generate")
            || content.contains("codegen")
            || content.contains("protoc")
            || content.contains("prisma generate")
            || content.contains("buf generate");

        if has_regen {
            self.pass()
        } else {
            self.fail(
                "CLAUDE.md doesn't document the code generation command",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Document codegen command".into(),
                    description: "Add the regeneration command to CLAUDE.md (protoc, prisma generate, npm run codegen, etc.). Without it, Claude skips regeneration after spec changes.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule GEN4: Auto-regeneration hook ───────────────────────────────

pub struct AutoRegenHook;

impl Rule for AutoRegenHook {
    fn id(&self) -> &str { "GEN4" }
    fn name(&self) -> &str { "PostToolUse hook auto-regenerates" }
    fn dimension(&self) -> Dimension { Dimension::Tooling }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_codegen(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.has_post_tool_use_hook_for_format() {
            // If any PostToolUse hooks exist, it's likely codegen is handled
            self.pass()
        } else {
            self.warn(
                "No PostToolUse hook for auto-regeneration after spec edits",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Add auto-regeneration hook".into(),
                    description: "Add a PostToolUse hook that detects spec file edits (.proto, .graphql, .prisma) and runs codegen automatically. Eliminates a whole class of type-mismatch bugs.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}
