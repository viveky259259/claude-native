mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::project_specific::doc_site::*;

#[test]
fn markdown_source_focus_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("docusaurus.config.js", "module.exports = {}"),
        ("package.json", r#"{"dependencies":{"@docusaurus/core":"3.0"}}"#),
        ("CLAUDE.md", "# Docs\nBuild: `npm run build`\nTest: `npm test`\nMarkdown source: docs/"),
        ("docs/intro.md", "# Welcome"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = MarkdownSourceFocus.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn navigation_documented_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("docusaurus.config.js", "module.exports = {}"),
        ("package.json", r#"{"dependencies":{"@docusaurus/core":"3.0"}}"#),
        ("CLAUDE.md", "# Docs\nBuild: `npm run build`\nTest: `npm test`\nTo add a page, update sidebars.js."),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = NavigationDocumented.check(&ctx);
    assert!(result.status.is_pass());
}
