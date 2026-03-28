mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::context::*;
use claude_native::rules::context_extra::*;

#[test]
fn no_mega_files_warns_approaching_threshold() {
    let content = "// line\n".repeat(350);
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/big.rs", &content),
        ("src/small.rs", "fn main() {}"),
    ]);
    let result = NoMegaFiles.check(&ctx);
    assert!(result.status.is_warning());
}

#[test]
fn generated_files_ignored_passes_with_claudeignore() {
    let (_dir, ctx) = helpers::scan_project(&[
        (".claudeignore", "dist/\nbuild/\n"),
        ("generated/api.pb.go", "// generated"),
    ]);
    let result = GeneratedFilesIgnored.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn subdir_claude_md_skips_small_projects() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", "fn main() {}"),
        ("src/lib.rs", "pub fn a() {}"),
    ]);
    let result = SubdirClaudeMd.check(&ctx);
    assert!(result.status.is_pass()); // <20 source files, not needed
}
