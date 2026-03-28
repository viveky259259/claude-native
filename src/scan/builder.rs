use std::path::{Path, PathBuf};

use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use walkdir::WalkDir;

use crate::scan::classifiers::*;
use crate::scan::{file_stats, FileInfo, ManifestInfo, ProjectContext};

/// Build a ProjectContext by scanning a project directory.
pub fn build_context(root: &Path) -> Result<ProjectContext> {
    let root = root.canonicalize()?;
    let scan = scan_directory(&root)?;
    let keys = read_key_files(&root);
    let claude = detect_claude_config(&root);
    let total_file_count = scan.all_files.len();

    Ok(ProjectContext {
        root, project_type: None,
        all_files: scan.all_files, total_file_count,
        max_depth: scan.max_depth, directories: scan.directories,
        claude_md_content: keys.claude_md_content, claude_md_path: keys.claude_md_path,
        claudeignore_content: keys.claudeignore_content,
        readme_content: keys.readme_content,
        settings_json: keys.settings_json, package_json: keys.package_json,
        package_manifests: scan.package_manifests,
        has_claude_dir: claude.has_dir, has_claude_rules_dir: claude.has_rules,
        has_claude_skills_dir: claude.has_skills,
        subdirectory_claude_mds: scan.subdirectory_claude_mds,
        test_files: scan.test_files, ci_configs: scan.ci_configs,
        env_files: scan.env_files, lock_files: scan.lock_files,
        mcp_json_path: claude.mcp_path,
        ignore_set: keys.ignore_set, ignore_patterns: keys.ignore_patterns,
        root_file_cache: keys.root_file_cache,
    })
}

struct ScanResult {
    all_files: Vec<FileInfo>, directories: Vec<PathBuf>,
    max_depth: usize, test_files: Vec<PathBuf>,
    ci_configs: Vec<PathBuf>, env_files: Vec<PathBuf>,
    lock_files: Vec<PathBuf>, package_manifests: Vec<ManifestInfo>,
    subdirectory_claude_mds: Vec<PathBuf>,
}

fn scan_directory(root: &Path) -> Result<ScanResult> {
    let mut r = ScanResult {
        all_files: vec![], directories: vec![], max_depth: 0,
        test_files: vec![], ci_configs: vec![], env_files: vec![],
        lock_files: vec![], package_manifests: vec![], subdirectory_claude_mds: vec![],
    };
    walk_directory(root, &mut r)?;
    Ok(r)
}

struct KeyFiles {
    claude_md_content: Option<String>, claude_md_path: Option<PathBuf>,
    claudeignore_content: Option<String>, readme_content: Option<String>,
    settings_json: Option<serde_json::Value>, package_json: Option<serde_json::Value>,
    ignore_set: Option<GlobSet>, ignore_patterns: Vec<String>,
    root_file_cache: std::collections::HashMap<String, String>,
}

fn read_key_files(root: &Path) -> KeyFiles {
    let mut root_file_cache = std::collections::HashMap::new();
    cache_root_files(root, &mut root_file_cache);
    let (claude_md_content, claude_md_path) = read_claude_md(root);
    let claudeignore_content = std::fs::read_to_string(root.join(".claudeignore")).ok();
    let (ignore_set, ignore_patterns) = build_ignore_set(&claudeignore_content);
    let readme_content = std::fs::read_to_string(root.join("README.md"))
        .or_else(|_| std::fs::read_to_string(root.join("readme.md"))).ok();
    let settings_json = std::fs::read_to_string(root.join(".claude/settings.json"))
        .ok().and_then(|s| serde_json::from_str(&s).ok());
    let package_json = std::fs::read_to_string(root.join("package.json"))
        .ok().and_then(|s| serde_json::from_str(&s).ok());
    KeyFiles { claude_md_content, claude_md_path, claudeignore_content, readme_content,
        settings_json, package_json, ignore_set, ignore_patterns, root_file_cache }
}

struct ClaudeConfig { has_dir: bool, has_rules: bool, has_skills: bool, mcp_path: Option<PathBuf> }

fn detect_claude_config(root: &Path) -> ClaudeConfig {
    let dir = root.join(".claude");
    ClaudeConfig {
        has_dir: dir.is_dir(),
        has_rules: dir.join("rules").is_dir(),
        has_skills: dir.join("skills").is_dir(),
        mcp_path: if dir.join(".mcp.json").exists() { Some(dir.join(".mcp.json")) } else { None },
    }
}

fn walk_directory(root: &Path, r: &mut ScanResult) -> Result<()> {
    let walker = WalkDir::new(root).follow_links(false).into_iter()
        .filter_entry(|e| !e.file_type().is_dir() || !should_skip_dir(e.file_name().to_str().unwrap_or("")));

    for entry in walker {
        let entry = entry?;
        let path = entry.path().to_path_buf();
        let depth = entry.depth();

        if entry.file_type().is_dir() {
            if depth > 0 { r.directories.push(path); }
            if depth > r.max_depth { r.max_depth = depth; }
            continue;
        }

        classify_file(&entry, root, depth, r);
    }
    Ok(())
}

fn classify_file(entry: &walkdir::DirEntry, root: &Path, depth: usize, r: &mut ScanResult) {
    let path = entry.path().to_path_buf();
    let file_name = entry.file_name().to_str().unwrap_or("");
    let relative_path = path.strip_prefix(root).unwrap_or(&path).to_path_buf();

    if is_lock_file(file_name) { r.lock_files.push(path.clone()); }
    if is_ci_config(&path) { r.ci_configs.push(path.clone()); }
    if is_env_file(file_name) { r.env_files.push(path.clone()); }
    if let Some(kind) = is_manifest(file_name) {
        r.package_manifests.push(ManifestInfo { path: path.clone(), kind });
    }
    if file_name == "CLAUDE.md" && depth > 0 {
        r.subdirectory_claude_mds.push(path.clone());
    }
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if is_source_extension(ext) || is_lock_file(file_name) || is_manifest(file_name).is_some() {
            let metadata = entry.metadata().unwrap_or_else(|_| std::fs::metadata(&path).unwrap());
            let is_test = is_test_file(&path);
            let is_generated = is_generated_file(&path);
            if is_test { r.test_files.push(path.clone()); }
            r.all_files.push(FileInfo {
                line_count: file_stats::count_lines(&path),
                size_bytes: metadata.len(), is_test, is_generated,
                path, relative_path,
            });
        }
    }
}

fn cache_root_files(root: &Path, cache: &mut std::collections::HashMap<String, String>) {
    let files_to_cache = [
        "Cargo.toml", "package.json", "go.mod", "pubspec.yaml",
        "requirements.txt", "pyproject.toml", "Gemfile", "mix.exs",
        "template.yaml", "template.yml", "config.toml",
    ];
    for filename in &files_to_cache {
        let path = root.join(filename);
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                cache.insert(filename.to_string(), content);
            }
        }
    }
}

/// Parse .claudeignore content into a compiled GlobSet and raw pattern list.
fn build_ignore_set(content: &Option<String>) -> (Option<GlobSet>, Vec<String>) {
    let content = match content {
        Some(c) => c,
        None => return (None, Vec::new()),
    };

    let mut builder = GlobSetBuilder::new();
    let mut patterns = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Skip negation patterns for now (advanced feature)
        if line.starts_with('!') {
            continue;
        }

        patterns.push(line.to_string());

        // Convert gitignore-style patterns to glob patterns:
        // "target/" → "**/target/**"  (directory anywhere)
        // "*.log"  → "**/*.log"       (extension anywhere)
        // "Cargo.lock" → "**/Cargo.lock" (file anywhere)
        let glob_pattern = if line.ends_with('/') {
            format!("**/{}/**", line.trim_end_matches('/'))
        } else if line.starts_with("**/") || line.starts_with('/') {
            line.to_string()
        } else if line.contains('/') {
            line.to_string()
        } else {
            format!("**/{line}")
        };

        // Try to compile the glob; skip invalid patterns silently
        if let Ok(glob) = Glob::new(&glob_pattern) {
            builder.add(glob);
        }
        // For directory patterns, also match the dir name itself
        if line.ends_with('/') {
            let dir = line.trim_end_matches('/');
            if let Ok(glob) = Glob::new(&format!("**/{dir}")) {
                builder.add(glob);
            }
        }
        // For bare names without extension, also treat as directory
        if !line.contains('.') && !line.ends_with('/') && !line.contains('*') {
            if let Ok(glob) = Glob::new(&format!("**/{line}/**")) {
                builder.add(glob);
            }
        }
    }

    let set = builder.build().ok();
    (set, patterns)
}

fn read_claude_md(root: &Path) -> (Option<String>, Option<PathBuf>) {
    let path1 = root.join("CLAUDE.md");
    if path1.exists() {
        if let Ok(content) = std::fs::read_to_string(&path1) {
            return (Some(content), Some(path1));
        }
    }
    let path2 = root.join(".claude").join("CLAUDE.md");
    if path2.exists() {
        if let Ok(content) = std::fs::read_to_string(&path2) {
            return (Some(content), Some(path2));
        }
    }
    (None, None)
}
