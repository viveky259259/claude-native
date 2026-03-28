mod helpers;

use claude_native::rules;
use claude_native::fix;

#[test]
fn fix_creates_missing_files() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt.clone());

    let results: Vec<_> = rules::all_rules().iter()
        .filter(|r| r.applies_to(&pt))
        .map(|r| r.check(&ctx))
        .collect();

    let actions = fix::apply_fixes(&ctx, &results).unwrap();
    assert!(!actions.is_empty(), "Should create files");
    assert!(ctx.root.join("CLAUDE.md").exists());
    assert!(ctx.root.join(".claudeignore").exists());
    assert!(ctx.root.join(".claude").join("settings.json").exists());
}

#[test]
fn fix_skips_existing_files() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Test\nBuild: `make`\nTest: `make test`"),
        ("AGENTS.md", "# Test\n## Build\n- Build: `make`"),
        (".claudeignore", "target/\nCargo.lock\n"),
        (".claude/settings.json", r#"{"permissions":{"allow":["Bash(cargo:*)"]}}"#),
        (".claude/rules/.gitkeep", ""),
        (".claude/skills/.gitkeep", ""),
        (".claude/.mcp.json", "{}"),
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt.clone());

    let results: Vec<_> = rules::all_rules().iter()
        .filter(|r| r.applies_to(&pt))
        .map(|r| r.check(&ctx))
        .collect();

    let actions = fix::apply_fixes(&ctx, &results).unwrap();
    // Should not recreate existing files
    let created_count = actions.iter().filter(|a| a.starts_with("Created")).count();
    assert_eq!(created_count, 0, "Should not overwrite existing files, got: {:?}", actions);
}
