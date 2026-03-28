mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::project_specific::serverless::*;

#[test]
fn deploy_artifacts_ignored_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("serverless.yml", "service: my-svc"),
        ("handler.js", "module.exports.hello = async () => {};"),
        (".claudeignore", ".serverless/\n"),
        (".serverless/state.json", "{}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = DeployArtifactsIgnored.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn functions_focused_passes_small_handlers() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("serverless.yml", "service: my-svc"),
        ("handler.js", &"// line\n".repeat(50)),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = FunctionsFocused.check(&ctx);
    assert!(result.status.is_pass());
}
