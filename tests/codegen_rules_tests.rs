mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::project_specific::codegen::*;

#[test]
fn generated_dirs_ignored_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("api.proto", "syntax = \"proto3\";"),
        ("generated/api.pb.go", "// generated"),
        (".claudeignore", "generated/\n*.pb.go\n"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = GeneratedDirsIgnored.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn auto_regen_hook_passes_with_hook() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("api.proto", "syntax = \"proto3\";"),
        ("generated/api.pb.go", "// generated"),
        (".claude/settings.json", r#"{"hooks": {"PostToolUse": [{"matcher": "Edit|Write", "hooks": []}]}}"#),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = AutoRegenHook.check(&ctx);
    assert!(result.status.is_pass());
}
