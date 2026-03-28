use crate::detection::{PrimaryType, ProjectType};
use crate::rules::*;
use crate::scan::ProjectContext;

fn is_doc_site(pt: &ProjectType) -> bool {
    matches!(pt.primary, PrimaryType::DocSite(_))
}

// ── Rule DOC1: Build output ignored ─────────────────────────────────

pub struct BuildOutputIgnored;

impl Rule for BuildOutputIgnored {
    fn id(&self) -> &str { "DOC1" }
    fn name(&self) -> &str { "Build output is ignored" }
    fn dimension(&self) -> Dimension { Dimension::ContextEfficiency }
    fn severity(&self) -> Severity { Severity::High }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_doc_site(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let outputs = ["build", "dist", "public", ".docusaurus", "site", "_site", "out"];
        let present: Vec<&&str> = outputs.iter().filter(|o| ctx.root.join(o).is_dir()).collect();

        if present.is_empty() {
            return self.pass();
        }

        let all_ignored = present.iter().all(|o| ctx.claudeignore_contains(o));
        if all_ignored {
            self.pass()
        } else {
            self.fail(
                &format!("Doc site build output dirs not ignored: {}", present.iter().map(|o| o.to_string()).collect::<Vec<_>>().join(", ")),
                Suggestion {
                    priority: SuggestionPriority::QuickWin,
                    title: "Ignore build output".into(),
                    description: "HTML output is 5-10x larger than markdown source. Add build/, dist/, .docusaurus/, site/ to .claudeignore.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule DOC2: Markdown source is the focus ─────────────────────────

pub struct MarkdownSourceFocus;

impl Rule for MarkdownSourceFocus {
    fn id(&self) -> &str { "DOC2" }
    fn name(&self) -> &str { "CLAUDE.md points to markdown source" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_doc_site(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.skip(),
        };

        let has_source_ref = content.contains("docs/")
            || content.contains("content/")
            || content.contains("markdown")
            || content.contains("source");

        if has_source_ref {
            self.pass()
        } else {
            self.warn(
                "CLAUDE.md doesn't point to the markdown source directory",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Document markdown source location".into(),
                    description: "Add to CLAUDE.md: 'Markdown source: docs/ (Claude reads this). Build output: build/ (don't read).'".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}

// ── Rule DOC3: Navigation config documented ─────────────────────────

pub struct NavigationDocumented;

impl Rule for NavigationDocumented {
    fn id(&self) -> &str { "DOC3" }
    fn name(&self) -> &str { "Navigation/sidebar config documented" }
    fn dimension(&self) -> Dimension { Dimension::Foundation }
    fn severity(&self) -> Severity { Severity::Medium }

    fn applies_to(&self, pt: &ProjectType) -> bool { is_doc_site(pt) }

    fn check(&self, ctx: &ProjectContext) -> RuleResult {
        let content = match &ctx.claude_md_content {
            Some(c) => c.to_lowercase(),
            None => return self.skip(),
        };

        let has_nav_docs = content.contains("sidebar")
            || content.contains("navigation")
            || content.contains("nav")
            || content.contains("menu");

        if has_nav_docs {
            self.pass()
        } else {
            self.warn(
                "CLAUDE.md doesn't explain how to add new pages to navigation",
                Suggestion {
                    priority: SuggestionPriority::NiceToHave,
                    title: "Document nav config".into(),
                    description: "Add to CLAUDE.md how to register new pages (which sidebar/nav file to update). Each doc framework has different config.".into(),
                    effort: Effort::Minutes,
                },
            )
        }
    }
}
