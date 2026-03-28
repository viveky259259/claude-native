mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::quality::*;
use claude_native::rules::quality_extra::*;

#[test]
fn type_annotations_pass_for_rust() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"test\""),
        ("src/main.rs", "fn main() {}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = TypeAnnotationsExist.check(&ctx);
    assert!(result.status.is_pass()); // Rust has built-in types
}

#[test]
fn type_annotations_pass_for_typescript() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("package.json", r#"{"name":"test"}"#),
        ("src/index.ts", "const x: number = 1;"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = TypeAnnotationsExist.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn tests_exist_warns_low_ratio() {
    let lines = "// line\n".repeat(20);
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/a.rs", &lines), ("src/b.rs", &lines), ("src/c.rs", &lines),
        ("src/d.rs", &lines), ("src/e.rs", &lines), ("src/f.rs", &lines),
        ("src/g.rs", &lines), ("src/h.rs", &lines), ("src/i.rs", &lines),
        ("src/j.rs", &lines), ("src/k.rs", &lines), ("src/l.rs", &lines),
        ("tests/a_test.rs", "#[test] fn t() {}"),
    ]);
    let result = TestsExist.check(&ctx);
    assert!(result.status.is_warning()); // 1 test for 12 source files
}

#[test]
fn consistent_patterns_passes_with_documented_patterns() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Style\nUse the repository pattern for data access.\nConvention: snake_case everywhere."),
        ("src/a.rs", ""), ("src/b.rs", ""), ("src/c.rs", ""),
        ("src/d.rs", ""), ("src/e.rs", ""), ("src/f.rs", ""),
        ("src/g.rs", ""), ("src/h.rs", ""), ("src/i.rs", ""),
        ("src/j.rs", ""), ("src/k.rs", ""),
    ]);
    let result = ConsistentPatterns.check(&ctx);
    assert!(result.status.is_pass());
}
