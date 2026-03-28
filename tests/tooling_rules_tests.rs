mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::tooling::*;

#[test]
fn mcp_servers_passes_with_config() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claude/.mcp.json", r#"{"mcpServers": {"github": {}}}"#),
    ]);
    let result = McpServersConfigured.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn mcp_servers_warns_without_config() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", ""),
    ]);
    let result = McpServersConfigured.check(&ctx);
    assert!(result.status.is_warning());
}

#[test]
fn dangerous_op_protection_passes_with_pre_hook() {
    let settings = r#"{"hooks": {"PreToolUse": [{"matcher": "Edit", "hooks": []}]}}"#;
    let (_dir, ctx) = helpers::scan_project(&[
        (".claude/settings.json", settings),
    ]);
    let result = DangerousOpProtection.check(&ctx);
    assert!(result.status.is_pass());
}
