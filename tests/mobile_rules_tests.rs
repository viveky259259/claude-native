mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::project_specific::mobile::*;

#[test]
fn binary_assets_excluded_passes_with_ignore() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("pubspec.yaml", "name: app"),
        ("lib/main.dart", "void main() {}"),
        ("assets/images/logo.png", ""),
        (".claudeignore", "*.png\n*.jpg\n*.svg\n"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = BinaryAssetsExcluded.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn lightweight_verification_passes_with_analyze() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("pubspec.yaml", "name: app"),
        ("lib/main.dart", "void main() {}"),
        ("CLAUDE.md", "# App\nLint: `flutter analyze`\nTest: `flutter test`"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = LightweightVerification.check(&ctx);
    assert!(result.status.is_pass());
}
