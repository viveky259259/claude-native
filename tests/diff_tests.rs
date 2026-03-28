mod helpers;

// diff::show_diff prints to stdout and uses temp dirs.
// We verify the underlying logic works via fix + score.

#[test]
fn diff_mode_compiles_and_runs() {
    // Just verify the diff module can be called without panic
    let dir = helpers::create_project(&[
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}"),
    ]);
    // We can't easily test stdout, but we can verify no panic
    let result = claude_native::diff::show_diff(dir.path());
    assert!(result.is_ok());
}
