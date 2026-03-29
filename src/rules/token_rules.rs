use crate::detection::ProjectType;
use crate::rules::*;
use crate::scan::ProjectContext;

// ═══════════════════════════════════════════════════════════════════
// R1: CLAUDE.md / README duplication check (suggestion only, no score)
// ═══════════════════════════════════════════════════════════════════

pub struct ClaudeMdReadmeDuplication;

impl Rule for ClaudeMdReadmeDuplication {
    fn id(&self) -> &str { "7.1" }
    fn name(&self) -> &str { "CLAUDE.md doesn't duplicate README" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let claude = match &ctx.claude_md_content {
            Some(c) => c,
            None => return self.skip(),
        };
        let readme = match &ctx.readme_content {
            Some(r) => r,
            None => return self.pass(),
        };

        let overlap = compute_line_overlap(claude, readme);
        if overlap > 0.3 {
            self.warn(
                &format!("{:.0}% of CLAUDE.md duplicates README content", overlap * 100.0),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Remove duplicated content from CLAUDE.md".into(),
                    description: "CLAUDE.md should contain ONLY what Claude can't infer:\n\
                        - Build/test commands\n\
                        - Code patterns and conventions\n\
                        - Gotchas and non-obvious behaviors\n\
                        - Architecture decisions\n\n\
                        Move project description, install guide, and usage to README only. \
                        Every duplicated line costs tokens on EVERY request.".into(),
                    effort: Effort::Minutes,
                },
            )
        } else {
            self.pass()
        }
    }
}

fn compute_line_overlap(a: &str, b: &str) -> f64 {
    let a_lines: Vec<&str> = a.lines()
        .map(|l| l.trim())
        .filter(|l| l.len() > 10) // skip short/empty lines
        .collect();
    if a_lines.is_empty() { return 0.0; }
    let b_content = b.to_lowercase();
    let matches = a_lines.iter()
        .filter(|l| b_content.contains(&l.to_lowercase()))
        .count();
    matches as f64 / a_lines.len() as f64
}

// ═══════════════════════════════════════════════════════════════════
// R2: Narrow .claude/rules/ path scopes
// ═══════════════════════════════════════════════════════════════════

pub struct NarrowRuleScopes;

impl Rule for NarrowRuleScopes {
    fn id(&self) -> &str { "7.2" }
    fn name(&self) -> &str { "Rule files use narrow path scopes" }
    fn dimension(&self) -> Dimension { Dimension::Tooling }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        if !ctx.has_claude_rules_dir { return self.skip(); }

        let rules_dir = ctx.root.join(".claude").join("rules");
        let mut broad_rules = Vec::new();

        if let Ok(entries) = std::fs::read_dir(&rules_dir) {
            for entry in entries.flatten() {
                if !entry.path().extension().map(|e| e == "md").unwrap_or(false) {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    if has_broad_scope(&content) {
                        let name = entry.file_name().to_string_lossy().to_string();
                        broad_rules.push(name);
                    }
                }
            }
        }

        if broad_rules.is_empty() {
            self.pass()
        } else {
            self.warn(
                &format!("Rules with broad paths: {}", broad_rules.join(", ")),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Narrow rule path scopes".into(),
                    description: "Use specific paths like `src/api/**` instead of `**/*.rs`. Broad scopes load the rule on every file access, wasting ~60 tokens per load.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

fn has_broad_scope(content: &str) -> bool {
    for line in content.lines() {
        let t = line.trim().trim_start_matches('-').trim();
        if t.starts_with("\"**/*") || t.starts_with("'**/*")
            || t == "\"**\"" || t == "'**'"
            || t.starts_with("\"src/**\"") || t.starts_with("'src/**'")
        {
            return true;
        }
    }
    false
}

// ═══════════════════════════════════════════════════════════════════
// R11: Targeted test command in CLAUDE.md
// ═══════════════════════════════════════════════════════════════════

pub struct TargetedTestCommand;

impl Rule for TargetedTestCommand {
    fn id(&self) -> &str { "7.3" }
    fn name(&self) -> &str { "Targeted test command documented" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Medium }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.skip(),
        };

        let has_targeted = content.contains("test single")
            || content.contains("test one")
            || content.contains("test specific")
            || content.contains("--test ")
            || content.contains("--testpathpattern")
            || content.contains("-t ")
            || content.contains("test module")
            || content.contains("test file")
            || content.contains("::"); // Rust module path for targeted tests

        if has_targeted {
            self.pass()
        } else {
            self.fail(
                "CLAUDE.md only has full test suite command, no targeted test command",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Add targeted test command to CLAUDE.md".into(),
                    description: "Add a command for testing single files/modules. This saves ~2000 tokens per test run.\n\
                        Examples:\n\
                        - Rust: `cargo test --test <name>` or `cargo test module::`\n\
                        - JS: `npm test -- --testPathPattern=<file>`\n\
                        - Python: `pytest tests/<file>.py`\n\
                        - Go: `go test ./pkg/<name>`".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// R12: Test output filtering hooks
// ═══════════════════════════════════════════════════════════════════

pub struct TestOutputFilteringHook;

impl Rule for TestOutputFilteringHook {
    fn id(&self) -> &str { "7.4" }
    fn name(&self) -> &str { "Test output filtering hook exists" }
    fn dimension(&self) -> Dimension { Dimension::Tooling }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let has_filter = ctx.settings_json.as_ref()
            .and_then(|v| v.get("hooks"))
            .map(|h| {
                let s = serde_json::to_string(h).unwrap_or_default().to_lowercase();
                s.contains("grep") || s.contains("fail") || s.contains("error")
                    || s.contains("filter") || s.contains("tail")
            })
            .unwrap_or(false);

        if has_filter {
            self.pass()
        } else {
            self.warn(
                "No hook to filter verbose test/build output",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Add test output filtering hook".into(),
                    description: "Add a PostToolUse hook that filters test output to failures only. \
                        Saves ~3000 tokens per test run.\n\
                        Example: grep -E '(FAIL|ERROR|panicked|test result:)' || echo 'All passed'".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// R14: Architecture decision records
// ═══════════════════════════════════════════════════════════════════

pub struct ArchDecisionRecords;

impl Rule for ArchDecisionRecords {
    fn id(&self) -> &str { "7.5" }
    fn name(&self) -> &str { "Architecture decision records exist" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::Low }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let has_adr = ctx.root.join("docs").join("adr").is_dir()
            || ctx.root.join("docs").join("decisions").is_dir()
            || ctx.root.join("adr").is_dir()
            || ctx.root.join("ADR").is_dir()
            || ctx.directories.iter().any(|d| {
                d.file_name().map(|n| n == "adr" || n == "decisions").unwrap_or(false)
            });

        // Also check if CLAUDE.md mentions architecture decisions
        let documented_in_claude = ctx.claude_md_content.as_ref().map(|c| {
            let l = c.to_lowercase();
            l.contains("decision") || l.contains("adr") || l.contains("why we")
        }).unwrap_or(false);

        if has_adr || documented_in_claude {
            self.pass()
        } else if ctx.source_file_count() < 20 {
            self.pass() // Small projects don't need ADRs
        } else {
            self.warn(
                "No architecture decision records found",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Add architecture decision records".into(),
                    description: "Create `docs/adr/` with markdown files documenting key decisions. \
                        Claude reads one ADR (~30 tokens) instead of reverse-engineering intent from code (~900 tokens).\n\
                        Template: docs/adr/001-use-redis-for-sessions.md\n\
                        Content: ## Decision, ## Reason, ## Alternatives Considered".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}
