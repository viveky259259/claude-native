mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::project_specific::polyglot::*;

#[test]
fn all_runtimes_ignored_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("go.mod", "module backend\ngo 1.21"),
        ("cmd/server/main.go", "package main"),
        ("package.json", r#"{"dependencies":{}}"#),
        ("frontend/src/App.tsx", "export default function App() {}"),
        (".claudeignore", "node_modules/\nvendor/\ntarget/\n"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = AllRuntimesIgnored.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn per_language_claude_md_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("go.mod", "module backend\ngo 1.21"),
        ("cmd/server/main.go", "package main"),
        ("package.json", r#"{"dependencies":{}}"#),
        ("frontend/src/App.tsx", "export default function App() {}"),
        ("backend/CLAUDE.md", "# Backend\nBuild: `go build`\nTest: `go test`"),
        ("frontend/CLAUDE.md", "# Frontend\nBuild: `npm run build`\nTest: `npm test`"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = PerLanguageClaudeMd.check(&ctx);
    assert!(result.status.is_pass());
}
