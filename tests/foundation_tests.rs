mod helpers;

use claude_native::rules::*;
use claude_native::rules::foundation::*;

// ── Rule 1.1: CLAUDE.md must exist ──────────────────────────────────

#[test]
fn claude_md_exists_passes_with_root_file() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Project\nBuild: `cargo build`\nTest: `cargo test`"),
        ("src/main.rs", "fn main() {}"),
    ]);
    let result = ClaudeMdExists.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn claude_md_exists_passes_with_dotclaude_file() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claude/CLAUDE.md", "# Project\nBuild: `make`"),
        ("src/main.rs", "fn main() {}"),
    ]);
    let result = ClaudeMdExists.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn claude_md_exists_fails_without_file() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", "fn main() {}"),
    ]);
    let result = ClaudeMdExists.check(&ctx);
    assert!(result.status.is_failure());
    assert_eq!(result.severity, Severity::Critical);
    assert!(result.suggestion.is_some());
}

// ── Rule 1.2: CLAUDE.md concise ─────────────────────────────────────

#[test]
fn claude_md_concise_passes_under_200() {
    let content = (0..50).map(|i| format!("- Rule {i}")).collect::<Vec<_>>().join("\n");
    let (_dir, ctx) = helpers::scan_project(&[
        ("CLAUDE.md", &content),
    ]);
    let result = ClaudeMdConcise.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn claude_md_concise_warns_200_to_400() {
    let content = (0..300).map(|i| format!("- Rule {i}")).collect::<Vec<_>>().join("\n");
    let (_dir, ctx) = helpers::scan_project(&[
        ("CLAUDE.md", &content),
    ]);
    let result = ClaudeMdConcise.check(&ctx);
    assert!(result.status.is_warning());
}

#[test]
fn claude_md_concise_fails_over_400() {
    let content = (0..500).map(|i| format!("- Rule {i}")).collect::<Vec<_>>().join("\n");
    let (_dir, ctx) = helpers::scan_project(&[
        ("CLAUDE.md", &content),
    ]);
    let result = ClaudeMdConcise.check(&ctx);
    assert!(result.status.is_failure());
}

#[test]
fn claude_md_concise_skips_when_missing() {
    let (_dir, ctx) = helpers::scan_project(&[("src/main.rs", "")]);
    let result = ClaudeMdConcise.check(&ctx);
    assert!(result.status.is_skip());
}

// ── Rule 1.4: CLAUDE.md has commands ────────────────────────────────

#[test]
fn claude_md_has_both_commands() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Project\nBuild: `cargo build`\nTest: `cargo test`"),
    ]);
    let result = ClaudeMdHasCommands.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn claude_md_missing_test_command() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Project\nBuild: `cargo build`"),
    ]);
    let result = ClaudeMdHasCommands.check(&ctx);
    assert!(result.status.is_warning());
}

#[test]
fn claude_md_no_commands() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# My Project\nThis is a project about things."),
    ]);
    let result = ClaudeMdHasCommands.check(&ctx);
    assert!(result.status.is_failure());
}

// ── Rule 1.5: .claudeignore exists ──────────────────────────────────

#[test]
fn claudeignore_exists_passes() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claudeignore", "node_modules/\ntarget/\n"),
    ]);
    let result = ClaudeignoreExists.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn claudeignore_exists_fails() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", "fn main() {}"),
    ]);
    let result = ClaudeignoreExists.check(&ctx);
    assert!(result.status.is_failure());
}

// ── Rule 1.6: .claude/ dir exists ───────────────────────────────────

#[test]
fn claude_dir_exists_passes() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claude/settings.json", "{}"),
    ]);
    let result = ClaudeDirExists.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn claude_dir_exists_fails() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", ""),
    ]);
    let result = ClaudeDirExists.check(&ctx);
    assert!(result.status.is_failure());
}

// ── Rule 1.7: settings.json with permissions ────────────────────────

#[test]
fn settings_with_permissions_passes() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claude/settings.json", r#"{"permissions": {"allow": ["Bash(cargo test:*)"]}}"#),
    ]);
    let result = SettingsJsonExists.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn settings_without_permissions_warns() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claude/settings.json", r#"{"hooks": {}}"#),
    ]);
    let result = SettingsJsonExists.check(&ctx);
    assert!(result.status.is_warning());
}

#[test]
fn settings_missing_fails() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", ""),
    ]);
    let result = SettingsJsonExists.check(&ctx);
    assert!(result.status.is_failure());
}
