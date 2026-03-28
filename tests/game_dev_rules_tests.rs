mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::project_specific::game_dev::*;

#[test]
fn game_logic_tests_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("project.godot", "[application]"),
        ("scripts/player.gd", "extends CharacterBody2D"),
        ("tests/player_test.py", "def test_movement(): pass"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = GameLogicTests.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn binary_assets_ignored_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("project.godot", "[application]"),
        (".claudeignore", "*.tscn\n*.tres\n*.png\n*.wav\n"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = BinaryAssetsIgnored.check(&ctx);
    assert!(result.status.is_pass());
}
