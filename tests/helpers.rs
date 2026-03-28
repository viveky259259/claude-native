use std::fs;

use tempfile::TempDir;

/// Create a temp project directory with specified files and their contents.
/// Files are specified as (relative_path, content) tuples.
pub fn create_project(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().expect("failed to create temp dir");
    for (path, content) in files {
        let full_path = dir.path().join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).expect("failed to create parent dirs");
        }
        fs::write(&full_path, content).expect("failed to write file");
    }
    dir
}

/// Create a temp project and scan it, returning the ProjectContext.
pub fn scan_project(files: &[(&str, &str)]) -> (TempDir, claude_native::scan::ProjectContext) {
    let dir = create_project(files);
    let ctx = claude_native::scan::build_context(dir.path()).expect("scan failed");
    (dir, ctx)
}

/// Create a project, scan it, detect type, and set project_type on context.
#[allow(dead_code)]
pub fn scan_and_detect(files: &[(&str, &str)]) -> (TempDir, claude_native::scan::ProjectContext) {
    let dir = create_project(files);
    let mut ctx = claude_native::scan::build_context(dir.path()).expect("scan failed");
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    (dir, ctx)
}
