use crate::rules::*;
use crate::scan::ProjectContext;

// ── Rule 1.8: AGENTS.md exists ──────────────────────────────────────

pub struct AgentsMdExists;

impl Rule for AgentsMdExists {
    fn id(&self) -> &str { "1.8" }
    fn name(&self) -> &str { "AGENTS.md exists (universal standard)" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.agents_md_content.is_some() {
            self.pass()
        } else {
            self.warn(
                "No AGENTS.md — the universal AI agent instruction standard",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Create AGENTS.md".into(),
                    description: "Run `claude-native --init` to generate AGENTS.md. Backed by Anthropic, OpenAI, Cursor, and Copilot.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}
