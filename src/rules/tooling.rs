use crate::rules::*;
use crate::scan::ProjectContext;

// ── Rule 4.1: MCP servers configured ────────────────────────────────

pub struct McpServersConfigured;

impl Rule for McpServersConfigured {
    fn id(&self) -> &str { "4.1" }
    fn name(&self) -> &str { "MCP servers configured" }
    fn dimension(&self) -> Dimension { Dimension::Tooling }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.mcp_json_path.is_some() {
            self.pass()
        } else {
            self.warn(
                "No .claude/.mcp.json found",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Configure MCP servers".into(),
                    description: "If your project uses external services (GitHub, databases, APIs), create .claude/.mcp.json to give Claude direct access. MCP tools are more reliable than shell workarounds.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule 4.2: Auto-format hook ──────────────────────────────────────

pub struct AutoFormatHook;

impl Rule for AutoFormatHook {
    fn id(&self) -> &str { "4.2" }
    fn name(&self) -> &str { "Hooks for auto-formatting" }
    fn dimension(&self) -> Dimension { Dimension::Tooling }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.has_post_tool_use_hook_for_format() {
            self.pass()
        } else {
            self.fail(
                "No PostToolUse hook for auto-formatting after edits",
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Add auto-format hook".into(),
                    description: "Add a PostToolUse hook in .claude/settings.json that runs your formatter (prettier, black, rustfmt, gofmt) after Edit/Write operations. This prevents style-related linter errors.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule 4.3: Dangerous operation protection ────────────────────────

pub struct DangerousOpProtection;

impl Rule for DangerousOpProtection {
    fn id(&self) -> &str { "4.3" }
    fn name(&self) -> &str { "Hooks for dangerous operation protection" }
    fn dimension(&self) -> Dimension { Dimension::Tooling }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.has_pre_tool_use_protection_hook() {
            self.pass()
        } else {
            self.warn(
                "No PreToolUse hooks for blocking dangerous operations",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Add protection hooks".into(),
                    description: "Add PreToolUse hooks to block editing sensitive files (.env, lock files, CI configs). Prevention is cheaper than correction.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule 4.4: Custom skills ─────────────────────────────────────────

pub struct CustomSkills;

impl Rule for CustomSkills {
    fn id(&self) -> &str { "4.4" }
    fn name(&self) -> &str { "Custom skills for workflows" }
    fn dimension(&self) -> Dimension { Dimension::Tooling }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.has_claude_skills_dir {
            self.pass()
        } else {
            self.warn(
                "No .claude/skills/ directory found",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Create custom skills".into(),
                    description: "Create .claude/skills/ with SKILL.md files for repetitive workflows (deploy, review, test). Skills load on-demand and encode complex multi-step processes.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule 4.5: Permission allow-list ─────────────────────────────────

pub struct PermissionAllowList;

impl Rule for PermissionAllowList {
    fn id(&self) -> &str { "4.5" }
    fn name(&self) -> &str { "Permission allow-list configured" }
    fn dimension(&self) -> Dimension { Dimension::Tooling }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.settings_has_permissions() {
            self.pass()
        } else {
            self.fail(
                "No permission allow-list in .claude/settings.json",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Configure permission allow-list".into(),
                    description: "Add permissions.allow to .claude/settings.json for safe commands (test runners, build tools, git). Every permission prompt interrupts Claude's flow.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule 4.6: Path-scoped rules ────────────────────────────────────

pub struct PathScopedRules;

impl Rule for PathScopedRules {
    fn id(&self) -> &str { "4.6" }
    fn name(&self) -> &str { ".claude/rules/ for path-scoped instructions" }
    fn dimension(&self) -> Dimension { Dimension::Tooling }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.has_claude_rules_dir {
            self.pass()
        } else {
            self.warn(
                "No .claude/rules/ directory found",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Create path-scoped rules".into(),
                    description: "Create .claude/rules/ with topic-specific .md files that use paths: frontmatter. Rules load only when Claude works with matching files, keeping CLAUDE.md small.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}
