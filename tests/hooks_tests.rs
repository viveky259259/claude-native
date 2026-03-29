mod helpers;

#[test]
fn hook_fails_without_git() {
    let dir = helpers::create_project(&[
        ("src/main.rs", "fn main() {}"),
    ]);
    // No .git directory — should fail
    let result = claude_native::hooks::install_hook(dir.path(), 70);
    assert!(result.is_err(), "Should fail without .git directory");
}

#[test]
fn hook_installs_with_git() {
    let dir = helpers::create_project(&[
        ("src/main.rs", "fn main() {}"),
    ]);
    // Create .git/hooks
    std::fs::create_dir_all(dir.path().join(".git/hooks")).unwrap();

    let result = claude_native::hooks::install_hook(dir.path(), 70);
    assert!(result.is_ok());

    let hook_path = dir.path().join(".git/hooks/pre-commit");
    assert!(hook_path.exists(), "Hook file should be created");

    let content = std::fs::read_to_string(&hook_path).unwrap();
    assert!(content.contains("claude-native"));
    assert!(content.contains("70")); // min score
}

#[test]
fn hook_contains_score_check() {
    let dir = helpers::create_project(&[]);
    std::fs::create_dir_all(dir.path().join(".git/hooks")).unwrap();

    claude_native::hooks::install_hook(dir.path(), 80).unwrap();
    let content = std::fs::read_to_string(dir.path().join(".git/hooks/pre-commit")).unwrap();
    assert!(content.contains("-lt 80"), "Should check against min score 80");
}
