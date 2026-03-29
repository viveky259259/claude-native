mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::drift::*;

#[test]
fn build_cmd_matches_passes_cargo() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("CLAUDE.md", "# Test\nBuild: `cargo build`\nTest: `cargo test`"),
        ("src/main.rs", "fn main() {}"),
    ]);
    let result = BuildCommandMatchesManifest.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn build_cmd_warns_on_mismatch() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("CLAUDE.md", "# Test\nBuild: `npm run build`\nTest: `npm test`"),
        ("src/main.rs", "fn main() {}"),
    ]);
    let result = BuildCommandMatchesManifest.check(&ctx);
    assert!(result.status.is_warning(), "npm commands with Cargo.toml should warn");
}

#[test]
fn build_cmd_skips_without_claude_md() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}"),
    ]);
    let result = BuildCommandMatchesManifest.check(&ctx);
    assert!(result.status.is_skip());
}

#[test]
fn referenced_files_passes_when_all_exist() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Test\nSee `src/main.rs` for entry point.\nBuild: `cargo build`"),
        ("src/main.rs", "fn main() {}"),
    ]);
    let result = ReferencedFilesExist.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn referenced_files_warns_on_missing() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Test\nSee `src/deleted_file.rs` for details.\nBuild: `cargo build`"),
        ("src/main.rs", "fn main() {}"),
    ]);
    let result = ReferencedFilesExist.check(&ctx);
    assert!(result.status.is_warning());
}

#[test]
fn referenced_files_ignores_commands() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Test\nBuild: `cargo build`\nTest: `cargo test`\nRun: `npm start`"),
        ("src/main.rs", "fn main() {}"),
    ]);
    let result = ReferencedFilesExist.check(&ctx);
    // "cargo build", "cargo test", "npm start" shouldn't be treated as file paths
    assert!(result.status.is_pass());
}
