mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::project_specific::monorepo::*;

#[test]
fn workspace_outputs_ignored_passes_with_wildcard() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("pnpm-workspace.yaml", "packages:\n  - packages/*"),
        (".claudeignore", "**/dist\n**/build\n**/node_modules\n"),
        ("packages/a/package.json", "{}"),
        ("packages/b/package.json", "{}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = WorkspaceOutputsIgnored.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn workspace_deps_navigable_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("pnpm-workspace.yaml", "packages:\n  - packages/*"),
        ("packages/a/package.json", r#"{"dependencies": {"b": "workspace:*"}}"#),
        ("packages/b/package.json", "{}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = WorkspaceDepsNavigable.check(&ctx);
    assert!(result.status.is_pass());
}
