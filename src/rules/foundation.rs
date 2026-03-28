use crate::rules::*;
use crate::scan::ProjectContext;

// ── Rule 1.1: CLAUDE.md must exist ──────────────────────────────────

pub struct ClaudeMdExists;

impl Rule for ClaudeMdExists {
    fn id(&self) -> &str { "1.1" }
    fn name(&self) -> &str { "CLAUDE.md must exist" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::Critical }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.has_claude_md() {
            self.pass()
        } else {
            self.fail(
                "No CLAUDE.md found at project root or .claude/CLAUDE.md",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Create a CLAUDE.md file".into(),
                    description: "Run `claude-native --init` to auto-generate CLAUDE.md with build/test commands for your detected project type. Or create manually with:\n  # Project Name\n  Build: `<your build cmd>`\n  Test: `<your test cmd>`\n  ## Code Patterns\n  <describe conventions>".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule 1.2: CLAUDE.md is concise ─────────────────────────────────

pub struct ClaudeMdConcise;

impl Rule for ClaudeMdConcise {
    fn id(&self) -> &str { "1.2" }
    fn name(&self) -> &str { "CLAUDE.md is concise (<200 lines)" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.claude_md_content.is_none() {
            return self.skip();
        }

        let lines = ctx.claude_md_line_count();
        if lines <= 200 {
            self.pass()
        } else if lines <= 400 {
            self.warn(
                &format!("CLAUDE.md is {lines} lines (target: <200). Every line costs tokens on every request."),
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Trim CLAUDE.md below 200 lines".into(),
                    description: "Move specialized instructions to .claude/rules/ (path-scoped, loaded on-demand) or .claude/skills/ (invoked explicitly). Keep only essentials in CLAUDE.md.".into(),
                    effort: Effort::Hour,
                },
            )
        } else {
            self.fail(
                &format!("CLAUDE.md is {lines} lines (target: <200). This wastes ~{} extra tokens per request.", (lines - 200) * 2),
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Significantly reduce CLAUDE.md".into(),
                    description: "Your CLAUDE.md is very long. Move domain-specific rules to .claude/rules/*.md with paths: frontmatter. Move workflows to .claude/skills/. Keep CLAUDE.md to: build/test commands, key conventions, gotchas.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule 1.3: CLAUDE.md contains actionable instructions ────────────

pub struct ClaudeMdActionable;

impl Rule for ClaudeMdActionable {
    fn id(&self) -> &str { "1.3" }
    fn name(&self) -> &str { "CLAUDE.md has actionable instructions" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c,
            None => return self.skip(),
        };
        let (has_actionable, prose_ratio) = analyze_actionability(content);
        if has_actionable {
            if prose_ratio > 0.6 {
                self.warn(
                    "CLAUDE.md seems too prose-heavy. It should focus on commands and rules, not explanations.",
                    Suggestion {
                        priority: SuggestionPriority::NiceToHave,
                        title: "Make CLAUDE.md more actionable".into(),
                        description: "Replace prose paragraphs with bullet points and code blocks. Claude needs commands and rules, not tutorials.".into(),
                        effort: Effort::Minutes,
                    },
                )
            } else {
                self.pass()
            }
        } else {
            self.fail(
                "CLAUDE.md appears to lack code blocks or runnable commands",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Add build/test commands to CLAUDE.md".into(),
                    description: "Add code-fenced commands that Claude can run: build, test, lint. Format as `command here`. Claude can't infer your project's specific commands.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

fn analyze_actionability(content: &str) -> (bool, f64) {
    let has_code = content.contains("```") || content.contains("    ");
    let has_cmds = content.contains('`') && ["npm ", "cargo ", "go ", "python ", "flutter ", "make ", "./"]
        .iter().any(|c| content.contains(c));
    let total = content.lines().count();
    let prose = content.lines().filter(|l| {
        let l = l.trim();
        !l.is_empty() && !l.starts_with('#') && !l.starts_with('-')
            && !l.starts_with('*') && !l.starts_with('`') && !l.starts_with("```")
    }).count();
    let ratio = if total > 0 { prose as f64 / total as f64 } else { 0.0 };
    (has_code || has_cmds, ratio)
}

// ── Rule 1.4: CLAUDE.md has build/test commands ─────────────────────

pub struct ClaudeMdHasCommands;

impl Rule for ClaudeMdHasCommands {
    fn id(&self) -> &str { "1.4" }
    fn name(&self) -> &str { "CLAUDE.md has build/test commands" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c,
            None => return self.skip(),
        };

        let lower = content.to_lowercase();
        let has_build = lower.contains("build")
            || lower.contains("compile")
            || lower.contains("make");
        let has_test = lower.contains("test")
            || lower.contains("spec")
            || lower.contains("check");

        if has_build && has_test {
            self.pass()
        } else if has_build || has_test {
            self.warn(
                &format!(
                    "CLAUDE.md is missing {} commands",
                    if has_build { "test" } else { "build" }
                ),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: format!("Add {} command to CLAUDE.md", if has_build { "test" } else { "build" }),
                    description: "Claude needs to know how to verify its own changes. Add both build and test commands.".into(),
                    effort: Effort::Minutes,
                },
            )
        } else {
            self.fail(
                "CLAUDE.md has no build or test commands",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Add build and test commands to CLAUDE.md".into(),
                    description: "Add lines like:\n  Build: `cargo build`\n  Test: `cargo test`\nClaude uses these to verify its changes work.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule 1.5: .claudeignore exists ──────────────────────────────────

pub struct ClaudeignoreExists;

impl Rule for ClaudeignoreExists {
    fn id(&self) -> &str { "1.5" }
    fn name(&self) -> &str { ".claudeignore exists and excludes noise" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.claudeignore_content.is_some() {
            self.pass()
        } else {
            self.fail(
                "No .claudeignore file found",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Create .claudeignore".into(),
                    description: "Run `claude-native --init` to auto-generate .claudeignore for your project type. Or create manually with:\n  node_modules/\n  .venv/\n  vendor/\n  dist/\n  build/\n  target/\n  coverage/\n  Cargo.lock\n  *.log\n  .env\n  .DS_Store".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule 1.6: .claude/ directory exists ─────────────────────────────

pub struct ClaudeDirExists;

impl Rule for ClaudeDirExists {
    fn id(&self) -> &str { "1.6" }
    fn name(&self) -> &str { ".claude/ directory exists" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.has_claude_dir {
            self.pass()
        } else {
            self.fail(
                "No .claude/ directory found",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Create .claude/ directory".into(),
                    description: "Create .claude/ at project root. This is the home for: settings.json (permissions), rules/ (path-scoped instructions), skills/ (custom workflows), and hooks.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule 1.7: .claude/settings.json with permissions ────────────────

pub struct SettingsJsonExists;

impl Rule for SettingsJsonExists {
    fn id(&self) -> &str { "1.7" }
    fn name(&self) -> &str { "settings.json has permissions" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if ctx.settings_json.is_none() {
            return self.fail(
                "No .claude/settings.json found",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Create .claude/settings.json".into(),
                    description: "Run `claude-native --init` to auto-generate settings.json. Or create manually:\n  {\"permissions\": {\"allow\": [\"Bash(cargo test:*)\", \"Bash(git:*)\"]}}\nPre-approved commands let Claude work without interrupting you.".into(),
                    effort: Effort::Minutes,
                },
            );
        }

        if ctx.settings_has_permissions() {
            self.pass()
        } else {
            self.warn(
                "settings.json exists but has no permission allow-list",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Add permissions to settings.json".into(),
                    description: "Add a permissions.allow list for common safe commands (test runners, build tools, git). This lets Claude work without interrupting you for every command.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// Rule 1.8 (AgentsMdExists) is in foundation_extra.rs
