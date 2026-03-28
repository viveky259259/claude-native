mod helpers;

// scan module tests

#[test]
fn context_has_correct_file_count() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", "fn main() {}"),
        ("src/lib.rs", "pub fn a() {}"),
        ("src/utils.rs", "pub fn b() {}"),
    ]);
    assert_eq!(ctx.all_files.len(), 3);
}

#[test]
fn context_reads_claude_md() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Test Project"),
    ]);
    assert!(ctx.claude_md_content.is_some());
    assert!(ctx.claude_md_content.unwrap().contains("Test Project"));
}

#[test]
fn context_reads_claudeignore() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claudeignore", "target/\nnode_modules/\n"),
    ]);
    assert!(ctx.claudeignore_content.is_some());
    assert!(ctx.claudeignore_contains("target"));
}

#[test]
fn context_reads_package_json() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("package.json", r#"{"name": "test", "version": "1.0.0"}"#),
    ]);
    assert!(ctx.package_json.is_some());
}

#[test]
fn context_reads_settings_json() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claude/settings.json", r#"{"permissions": {"allow": ["Bash(npm:*)"]}}"#),
    ]);
    assert!(ctx.settings_json.is_some());
    assert!(ctx.settings_has_permissions());
}

#[test]
fn context_detects_claude_subdirs() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claude/rules/api.md", "# API rules"),
        (".claude/skills/deploy/SKILL.md", "# Deploy"),
    ]);
    assert!(ctx.has_claude_dir);
    assert!(ctx.has_claude_rules_dir);
    assert!(ctx.has_claude_skills_dir);
}

#[test]
fn context_finds_subdirectory_claude_mds() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Root"),
        ("api/CLAUDE.md", "# API"),
        ("web/CLAUDE.md", "# Web"),
    ]);
    // Subdirectory CLAUDE.md files are those at depth > 0
    assert!(ctx.subdirectory_claude_mds.len() >= 2,
        "Expected ≥2 subdirectory CLAUDE.md files, found {}", ctx.subdirectory_claude_mds.len());
}

#[test]
fn context_finds_mcp_json() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claude/.mcp.json", r#"{"mcpServers": {}}"#),
    ]);
    assert!(ctx.mcp_json_path.is_some());
}

#[test]
fn claudeignored_file_excluded_from_mega_files() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claudeignore", "big_file.rs\n"),
        ("src/big_file.rs", &"// line\n".repeat(600)),
        ("src/small.rs", "fn main() {}"),
    ]);
    let mega = ctx.mega_files(500);
    assert!(mega.is_empty(), "File in .claudeignore should not appear in mega_files");
}

#[test]
fn is_claudeignored_works() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claudeignore", "target/\nCargo.lock\n*.log\n"),
    ]);
    assert!(ctx.is_claudeignored("target/debug/build"));
    assert!(ctx.is_claudeignored("Cargo.lock"));
    assert!(!ctx.is_claudeignored("src/main.rs"));
}

#[test]
fn context_line_count_is_accurate() {
    let content = "line 1\nline 2\nline 3\nline 4\nline 5\n";
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/five_lines.rs", content),
    ]);
    let file = ctx.all_files.iter().find(|f| f.relative_path.to_string_lossy().contains("five_lines")).unwrap();
    assert_eq!(file.line_count, 5);
}

#[test]
fn average_source_file_lines_excludes_tests() {
    let big = &"// line\n".repeat(100);
    let small = &"// line\n".repeat(10);
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/big.rs", big),
        ("src/small.rs", small),
        ("tests/huge_test.rs", &"// line\n".repeat(500)),
    ]);
    let avg = ctx.average_source_file_lines();
    assert!(avg < 100.0, "Average should exclude test files, got {avg}");
}
