mod helpers;

#[test]
fn init_creates_missing_files() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);

    let created = claude_native::init::init_project(&ctx).unwrap();
    assert!(created.contains(&"CLAUDE.md".to_string()));
    assert!(created.contains(&".claudeignore".to_string()));
    assert!(created.contains(&".claude/settings.json".to_string()));

    // Verify files exist
    assert!(ctx.root.join("CLAUDE.md").exists());
    assert!(ctx.root.join(".claudeignore").exists());
    assert!(ctx.root.join(".claude/settings.json").exists());
}

#[test]
fn init_skips_existing_files() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Existing"),
        (".claudeignore", "target/"),
        (".claude/settings.json", "{}"),
        ("src/main.rs", "fn main() {}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);

    let created = claude_native::init::init_project(&ctx).unwrap();
    assert!(created.is_empty(), "Should not overwrite existing files");
}

#[test]
fn init_generates_correct_commands_for_rust() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);

    claude_native::init::init_project(&ctx).unwrap();
    let content = std::fs::read_to_string(ctx.root.join("CLAUDE.md")).unwrap();
    assert!(content.contains("cargo build"), "Should have cargo build command");
    assert!(content.contains("cargo test"), "Should have cargo test command");
}
