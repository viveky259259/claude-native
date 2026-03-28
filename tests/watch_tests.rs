// watch::watch_and_score runs an infinite loop, so we can't test it directly.
// Instead we test the underlying scoring function works correctly.

mod helpers;

#[test]
fn score_can_be_computed_repeatedly() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Test\nBuild: `cargo build`\nTest: `cargo test`"),
        ("src/main.rs", "fn main() {}"),
        ("Cargo.toml", "[package]\nname = \"test\""),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt.clone());

    // Score twice to verify no state mutation issues
    for _ in 0..2 {
        let results: Vec<_> = claude_native::rules::all_rules().iter()
            .filter(|r| r.applies_to(&pt))
            .map(|r| r.check(&ctx))
            .collect();
        let sc = claude_native::scoring::calculate(results, &pt);
        assert!(sc.total_score > 0.0);
    }
}
