mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::navigation::*;
use claude_native::rules::navigation_extra::*;

#[test]
fn consistent_naming_passes_all_snake_case() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main_handler.rs", ""),
        ("src/auth_service.rs", ""),
        ("src/user_model.rs", ""),
        ("src/data_store.rs", ""),
        ("src/cache_layer.rs", ""),
        ("src/error_types.rs", ""),
    ]);
    let result = ConsistentNaming.check(&ctx);
    assert!(result.status.is_pass(), "All snake_case should pass");
}

#[test]
fn consistent_naming_fails_mixed() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main_handler.rs", ""),
        ("src/AuthService.rs", ""),
        ("src/user-model.rs", ""),
        ("src/dataStore.rs", ""),
        ("src/cache_layer.rs", ""),
        ("src/ErrorTypes.rs", ""),
    ]);
    let result = ConsistentNaming.check(&ctx);
    assert!(result.status.is_failure() || result.status.is_warning());
}

#[test]
fn predictable_test_locations_passes_co_located() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/auth.rs", ""),
        ("src/auth_test.rs", "#[test] fn t() {}"),
        ("src/user.rs", ""),
        ("src/user_test.rs", "#[test] fn t() {}"),
    ]);
    let result = PredictableTestLocations.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn clear_structure_fails_too_many_root_files() {
    let files: Vec<(String, String)> = (0..20)
        .map(|i| (format!("file_{i}.rs"), format!("fn f_{i}() {{}}")))
        .collect();
    let file_refs: Vec<(&str, &str)> = files.iter()
        .map(|(p, c)| (p.as_str(), c.as_str()))
        .collect();
    let (_dir, ctx) = helpers::scan_project(&file_refs);
    let result = ClearDirectoryStructure.check(&ctx);
    assert!(result.status.is_failure() || result.status.is_warning());
}
