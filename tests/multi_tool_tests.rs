mod helpers;

use claude_native::multi_tool;

#[test]
fn generate_all_creates_cursorrules_and_copilot() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);

    let created = multi_tool::generate_all(&ctx).unwrap();
    assert!(created.contains(&".cursorrules".to_string()), "Should create .cursorrules");
    assert!(created.contains(&".github/copilot-instructions.md".to_string()), "Should create copilot-instructions.md");

    // Verify files exist
    assert!(ctx.root.join(".cursorrules").exists());
    assert!(ctx.root.join(".github/copilot-instructions.md").exists());
}

#[test]
fn generate_all_includes_init_files() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);

    let created = multi_tool::generate_all(&ctx).unwrap();
    assert!(created.contains(&"CLAUDE.md".to_string()));
    assert!(created.contains(&"AGENTS.md".to_string()));
    assert!(created.contains(&".claudeignore".to_string()));
}

#[test]
fn generate_all_skips_existing() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}"),
        ("CLAUDE.md", "# Existing"),
        ("AGENTS.md", "# Existing"),
        (".claudeignore", "target/"),
        (".claude/settings.json", "{}"),
        (".cursorrules", "# Existing"),
        (".github/copilot-instructions.md", "# Existing"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);

    let created = multi_tool::generate_all(&ctx).unwrap();
    assert!(created.is_empty(), "Should not overwrite existing files");
}

#[test]
fn cursorrules_contains_build_commands() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);

    multi_tool::generate_all(&ctx).unwrap();
    let content = std::fs::read_to_string(ctx.root.join(".cursorrules")).unwrap();
    assert!(content.contains("cargo build"));
    assert!(content.contains("cargo test"));
}
