mod helpers;

use claude_native::rules::*;
use claude_native::rules::context::*;
use claude_native::rules::context_extra::*;
use claude_native::rules::navigation::*;
use claude_native::rules::navigation_extra::*;
use claude_native::rules::tooling::*;
use claude_native::rules::quality::*;
use claude_native::rules::quality_extra::*;

// ═══════════════════════════════════════════════════════════════════
// Context Efficiency rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn no_mega_files_passes_when_all_small() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", &"// line\n".repeat(100)),
        ("src/lib.rs", &"// line\n".repeat(50)),
    ]);
    let result = NoMegaFiles.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn no_mega_files_fails_with_large_file() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", &"// line\n".repeat(600)),
    ]);
    let result = NoMegaFiles.check(&ctx);
    assert!(result.status.is_failure());
}

#[test]
fn lock_files_ignored_passes_with_claudeignore() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("Cargo.lock", "# lock content"),
        (".claudeignore", "Cargo.lock\npackage-lock.json\n"),
    ]);
    let result = LockFilesIgnored.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn lock_files_ignored_fails_without_claudeignore() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("Cargo.lock", "# lock content"),
    ]);
    let result = LockFilesIgnored.check(&ctx);
    assert!(result.status.is_failure());
}

#[test]
fn no_secrets_passes_with_only_example_env() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".env.example", "API_KEY=xxx"),
    ]);
    let result = NoSecretsInRepo.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn no_secrets_fails_with_real_env() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".env", "API_KEY=secret123"),
    ]);
    let result = NoSecretsInRepo.check(&ctx);
    assert!(result.status.is_failure());
    assert_eq!(result.severity, Severity::Critical);
}

#[test]
fn readme_passes_when_concise() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("README.md", &"# My Project\n".repeat(50)),
    ]);
    let result = ReadmeExistsAndConcise.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn readme_fails_when_missing() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", ""),
    ]);
    let result = ReadmeExistsAndConcise.check(&ctx);
    assert!(result.status.is_failure());
}

#[test]
fn readme_warns_when_too_long() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("README.md", &"# Section\nLong content here.\n".repeat(200)),
    ]);
    let result = ReadmeExistsAndConcise.check(&ctx);
    assert!(result.status.is_warning());
}

// ═══════════════════════════════════════════════════════════════════
// Navigation rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn no_deep_nesting_passes() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/mod/file.rs", ""),
        ("src/mod/sub/file.rs", ""),
    ]);
    let result = NoDeepNesting.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn no_deep_nesting_warns_when_deep() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("a/b/c/d/e/f/deep.rs", "fn main() {}"),
    ]);
    let result = NoDeepNesting.check(&ctx);
    assert!(result.status.is_warning());
}

#[test]
fn obvious_entry_points_passes_with_main() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", "fn main() {}"),
    ]);
    let result = ObviousEntryPoints.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn obvious_entry_points_passes_with_index() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/index.ts", "export default {};"),
    ]);
    let result = ObviousEntryPoints.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn descriptive_names_passes() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/handlers/auth.rs", ""),
        ("src/models/user.rs", ""),
    ]);
    let result = DescriptiveNames.check(&ctx);
    assert!(result.status.is_pass());
}

// ═══════════════════════════════════════════════════════════════════
// Tooling rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn permission_allow_list_passes() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claude/settings.json", r#"{"permissions": {"allow": ["Bash(cargo test:*)"]}}"#),
    ]);
    let result = PermissionAllowList.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn permission_allow_list_fails_without_settings() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", ""),
    ]);
    let result = PermissionAllowList.check(&ctx);
    assert!(result.status.is_failure());
}

#[test]
fn auto_format_hook_passes_with_hook() {
    let settings = r#"{
        "hooks": {
            "PostToolUse": [{"matcher": "Edit|Write", "hooks": [{"type": "command", "command": "rustfmt"}]}]
        }
    }"#;
    let (_dir, ctx) = helpers::scan_project(&[
        (".claude/settings.json", settings),
    ]);
    let result = AutoFormatHook.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn auto_format_hook_fails_without_hook() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claude/settings.json", r#"{"permissions": {}}"#),
    ]);
    let result = AutoFormatHook.check(&ctx);
    assert!(result.status.is_failure());
}

#[test]
fn path_scoped_rules_passes_with_dir() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claude/rules/backend.md", "---\npaths: \"src/api/**\"\n---\n# API rules"),
    ]);
    let result = PathScopedRules.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn custom_skills_passes_with_dir() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claude/skills/deploy/SKILL.md", "# Deploy\n1. Build\n2. Push"),
    ]);
    let result = CustomSkills.check(&ctx);
    assert!(result.status.is_pass());
}

// ═══════════════════════════════════════════════════════════════════
// Code Quality rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn tests_exist_passes() {
    let lines = "// line\n".repeat(20);
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/lib.rs", &lines),
        ("src/utils.rs", &lines),
        ("src/handler.rs", &lines),
        ("tests/lib_test.rs", "#[test] fn it_works() {}"),
    ]);
    let result = TestsExist.check(&ctx);
    assert!(result.status.is_pass() || result.status.is_warning());
}

#[test]
fn tests_exist_fails_without_tests() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", "fn main() {}"),
        ("src/lib.rs", "pub fn hello() {}"),
    ]);
    let result = TestsExist.check(&ctx);
    assert!(result.status.is_failure());
}

#[test]
fn dependencies_documented_passes_with_cargo() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname=\"test\""),
        ("src/main.rs", "fn main() {}"),
    ]);
    let result = DependenciesDocumented.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn dependencies_documented_passes_with_package_json() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("package.json", r#"{"name": "test"}"#),
        ("index.js", ""),
    ]);
    let result = DependenciesDocumented.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn ci_exists_passes_with_github_actions() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".github/workflows/ci.yml", "name: CI\non: [push]"),
    ]);
    let result = CiCdExists.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn ci_exists_warns_without_config() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", ""),
    ]);
    let result = CiCdExists.check(&ctx);
    assert!(result.status.is_warning());
}

#[test]
fn no_dead_code_passes_clean_project() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", "fn main() {}"),
        ("src/lib.rs", "pub fn hello() {}"),
    ]);
    let result = NoDeadCode.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn no_dead_code_warns_with_deprecated_dir() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", "fn main() {}"),
        ("deprecated/old.rs", "// old code"),
    ]);
    let result = NoDeadCode.check(&ctx);
    assert!(result.status.is_warning());
}
