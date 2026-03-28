use crate::detection::ProjectType;
use crate::rules::*;
use crate::scan::ProjectContext;

fn is_legacy(pt: &ProjectType) -> bool {
    pt.flags.is_legacy
}

// ── Rule LEG1: CLAUDE.md documents the mess ─────────────────────────

pub struct DocumentsTheMess;

impl Rule for DocumentsTheMess {
    fn id(&self) -> &str { "LEG1" }
    fn name(&self) -> &str { "CLAUDE.md documents known inconsistencies" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_legacy(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.fail(
                "Legacy project has no CLAUDE.md — Claude is working blind in a messy codebase",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Create CLAUDE.md documenting the mess".into(),
                    description: "Legacy projects NEED CLAUDE.md more than any other type. Document: known inconsistencies, which patterns are 'correct' vs 'legacy', dead code directories, and the preferred approach for changes.".into(),
                    effort: Effort::Hour,
                },
            ),
        };

        let documents_issues = content.contains("legacy")
            || content.contains("deprecated")
            || content.contains("inconsisten")
            || content.contains("old pattern")
            || content.contains("do not use")
            || content.contains("don't use")
            || content.contains("prefer");

        if documents_issues {
            self.pass()
        } else {
            self.warn(
                "CLAUDE.md doesn't mention legacy patterns or inconsistencies",
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Document legacy patterns in CLAUDE.md".into(),
                    description: "Add a section listing: which patterns are correct vs legacy, dead code directories, and the incremental approach. Without this, Claude picks the wrong pattern 50% of the time.".into(),
                    effort: Effort::Hour,
                },
            )
        }
    }
}

// ── Rule LEG2: Correct patterns are identified ──────────────────────

pub struct CorrectPatternsIdentified;

impl Rule for CorrectPatternsIdentified {
    fn id(&self) -> &str { "LEG2" }
    fn name(&self) -> &str { "Correct patterns explicitly identified" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_legacy(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.skip(),
        };

        // Check for pattern references with file paths
        let has_pattern_refs = content.contains("follow ")
            || content.contains("use the pattern in")
            || content.contains("reference:")
            || (content.contains("not ") && content.contains("pattern"));

        if has_pattern_refs {
            self.pass()
        } else {
            self.fail(
                "CLAUDE.md doesn't explicitly identify which patterns are correct",
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Identify correct patterns with file references".into(),
                    description: "Add to CLAUDE.md:\n- Error handling: follow `src/services/auth.ts` (NOT `src/handlers/legacy.ts`)\n- Data access: use `src/db/repository.ts` pattern\nClaude needs ONE canonical reference per concern.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule LEG3: Tests exist for modified code ────────────────────────

pub struct TestsForModifiedCode;

impl Rule for TestsForModifiedCode {
    fn id(&self) -> &str { "LEG3" }
    fn name(&self) -> &str { "Tests exist for code being modified" }
    fn dimension(&self) -> Dimension { Dimension::CodeQuality }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_legacy(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        // In legacy projects, any tests are a win
        if ctx.test_files.is_empty() {
            self.fail(
                "Legacy project has ZERO tests — changes can't be verified",
                Suggestion {
                    priority: SuggestionPriority::HighImpact,
                    title: "Add tests for critical modules".into(),
                    description: "Write tests for the specific modules you're modifying. Claude should write a test FIRST, then make changes. Full coverage is unrealistic in legacy — start with critical paths.".into(),
                    effort: Effort::HalfDay,
                },
            )
        } else {
            self.pass()
        }
    }
}

// ── Rule LEG4: Dead code is flagged ─────────────────────────────────

pub struct DeadCodeFlagged;

impl Rule for DeadCodeFlagged {
    fn id(&self) -> &str { "LEG4" }
    fn name(&self) -> &str { "Dead code directories are flagged" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_legacy(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let dead_dirs = ["deprecated", "old", "backup", "archive", "legacy", "unused"];
        let found: Vec<String> = ctx.directories.iter()
            .filter_map(|d| {
                let name = d.file_name()?.to_str()?.to_lowercase();
                if dead_dirs.contains(&name.as_str()) { Some(name) } else { None }
            })
            .collect();

        if found.is_empty() {
            return self.pass();
        }

        // Check if they're documented in CLAUDE.md
        let documented = ctx.claude_md_content.as_ref().map(|c| {
            let lower = c.to_lowercase();
            found.iter().any(|d| lower.contains(d))
        }).unwrap_or(false);

        if documented {
            self.pass()
        } else {
            self.warn(
                &format!("Dead code directories exist ({}) but aren't documented in CLAUDE.md", found.join(", ")),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Document dead code in CLAUDE.md".into(),
                    description: format!("Add to CLAUDE.md:\n# Dead code (do not use)\n{}\nThis prevents Claude from reading/using deprecated code.", found.iter().map(|d| format!("- {d}/")).collect::<Vec<_>>().join("\n")),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule LEG5: Mega-files are documented ────────────────────────────

pub struct MegaFilesDocumented;

impl Rule for MegaFilesDocumented {
    fn id(&self) -> &str { "LEG5" }
    fn name(&self) -> &str { "Large legacy files have line-range docs" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_legacy(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let mega = ctx.mega_files(500);
        if mega.is_empty() {
            return self.pass();
        }

        let documented = ctx.claude_md_content.as_ref().map(|c| {
            c.contains("L") && (c.contains("-") || c.contains("lines"))
        }).unwrap_or(false);

        if documented {
            self.pass()
        } else {
            let examples: Vec<String> = mega.iter().take(3)
                .map(|f| format!("{} ({} lines)", f.relative_path.display(), f.line_count))
                .collect();
            self.warn(
                &format!("Large legacy files exist without line-range documentation:\n  {}", examples.join("\n  ")),
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Document line ranges in mega-files".into(),
                    description: format!("Add to CLAUDE.md:\n# Large files\n{}\nTell Claude where key logic lives so it doesn't read the entire file.", examples.iter().map(|e| format!("- {e} — auth: L200-350, routing: L400-600")).collect::<Vec<_>>().join("\n")),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}
