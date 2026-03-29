mod helpers;

use claude_native::{detection, rules, scoring, history};

#[test]
fn history_creates_file_and_records_entry() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Test\nBuild: `cargo build`\nTest: `cargo test`"),
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}"),
    ]);
    let pt = detection::detect(&ctx);
    ctx.project_type = Some(pt.clone());

    let results: Vec<_> = rules::all_rules().iter()
        .filter(|r| r.applies_to(&pt))
        .map(|r| r.check(&ctx))
        .collect();
    let sc = scoring::calculate(results, &pt);

    history::record_and_show(&sc, &ctx.root).unwrap();

    let history_path = ctx.root.join(".claude-native-history.json");
    assert!(history_path.exists(), "History file should be created");

    let content = std::fs::read_to_string(&history_path).unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0]["score"].as_f64().unwrap() > 0.0);
    assert!(entries[0]["grade"].as_str().is_some());
}

#[test]
fn history_appends_to_existing() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Test\nBuild: `cargo build`\nTest: `cargo test`"),
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}"),
    ]);
    let pt = detection::detect(&ctx);
    ctx.project_type = Some(pt.clone());

    let results: Vec<_> = rules::all_rules().iter()
        .filter(|r| r.applies_to(&pt))
        .map(|r| r.check(&ctx))
        .collect();
    let sc = scoring::calculate(results, &pt);

    // Record twice
    history::record_and_show(&sc, &ctx.root).unwrap();
    history::record_and_show(&sc, &ctx.root).unwrap();

    let content = std::fs::read_to_string(ctx.root.join(".claude-native-history.json")).unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap();
    assert_eq!(entries.len(), 2, "Should have 2 entries");
}
