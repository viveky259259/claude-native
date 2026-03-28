
use crate::detection::Language;
use crate::scan::ProjectContext;

// ── Language detection ──────────────────────────────────────────────

pub fn detect_languages(ctx: &ProjectContext) -> Vec<Language> {
    let mut found = std::collections::HashSet::new();
    for file in &ctx.all_files {
        if let Some(lang) = ext_to_language(file.path.extension().and_then(|e| e.to_str())) {
            found.insert(lang);
        }
    }
    // JS only if no TS (TS supersedes JS)
    if found.contains("ts") { found.remove("js"); }
    found.into_iter().filter_map(lang_from_key).collect()
}

fn ext_to_language(ext: Option<&str>) -> Option<&'static str> {
    match ext? {
        "rs" => Some("rs"), "ts" | "tsx" => Some("ts"),
        "js" | "jsx" | "mjs" | "cjs" => Some("js"), "py" => Some("py"),
        "go" => Some("go"), "dart" => Some("dart"), "swift" => Some("swift"),
        "kt" | "kts" => Some("kt"), "java" => Some("java"), "rb" => Some("rb"),
        "cs" => Some("cs"), "ex" | "exs" => Some("ex"),
        "cpp" | "cc" | "cxx" | "c" | "h" | "hpp" => Some("cpp"),
        _ => None,
    }
}

fn lang_from_key(key: &str) -> Option<Language> {
    match key {
        "rs" => Some(Language::Rust), "ts" => Some(Language::TypeScript),
        "js" => Some(Language::JavaScript), "py" => Some(Language::Python),
        "go" => Some(Language::Go), "dart" => Some(Language::Dart),
        "swift" => Some(Language::Swift), "kt" => Some(Language::Kotlin),
        "java" => Some(Language::Java), "rb" => Some(Language::Ruby),
        "cs" => Some(Language::CSharp), "ex" => Some(Language::Elixir),
        "cpp" => Some(Language::Cpp), _ => None,
    }
}

// ── Structure detection ─────────────────────────────────────────────

pub fn is_monorepo(ctx: &ProjectContext) -> bool {
    // Multiple package manifests at different directory levels
    let manifest_dirs: Vec<_> = ctx.package_manifests.iter()
        .filter_map(|m| m.path.parent())
        .collect();
    let unique_dirs: std::collections::HashSet<_> = manifest_dirs.iter().collect();
    if unique_dirs.len() > 2 {
        return true;
    }

    // Workspace config files
    let workspace_files = [
        "pnpm-workspace.yaml",
        "lerna.json",
        "nx.json",
        "turbo.json",
        "go.work",
    ];
    for wf in &workspace_files {
        if ctx.has_file(wf) {
            return true;
        }
    }

    // Check Cargo.toml for [workspace]
    if let Some(cargo) = ctx.read_manifest_content("Cargo.toml") {
        if cargo.contains("[workspace]") {
            return true;
        }
    }

    // Top-level workspace directories
    let workspace_dirs = ["packages", "apps", "services", "libs"];
    for wd in &workspace_dirs {
        let dir = ctx.root.join(wd);
        if dir.is_dir() {
            // Check it has subdirectories (not just an empty folder)
            if let Ok(entries) = std::fs::read_dir(&dir) {
                let subdir_count = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .count();
                if subdir_count >= 2 {
                    return true;
                }
            }
        }
    }

    false
}

pub fn is_micro_repo(ctx: &ProjectContext) -> bool {
    ctx.package_manifests.len() <= 1
        && ctx.source_file_count() < 50
        && ctx.max_depth <= 3
        && !is_monorepo(ctx)
}

// ── Mobile signals ──────────────────────────────────────────────────

pub fn has_react_native_dep(ctx: &ProjectContext) -> bool {
    package_json_has_dep(ctx, "react-native")
}

pub fn has_xcodeproj(ctx: &ProjectContext) -> bool {
    ctx.directories.iter().any(|d| {
        d.extension()
            .map(|e| e == "xcodeproj")
            .unwrap_or(false)
    })
}

pub fn has_android_gradle(ctx: &ProjectContext) -> bool {
    // build.gradle.kts or build.gradle at android/ or root with android plugin
    let android_dir = ctx.root.join("android");
    if android_dir.join("build.gradle.kts").exists() || android_dir.join("build.gradle").exists() {
        return true;
    }
    let root_gradle = ctx.root.join("build.gradle.kts");
    if root_gradle.exists() {
        if let Ok(content) = std::fs::read_to_string(&root_gradle) {
            return content.contains("com.android.application") || content.contains("com.android.library");
        }
    }
    false
}

// ── Game dev signals ────────────────────────────────────────────────

pub fn has_unity(ctx: &ProjectContext) -> bool {
    ctx.all_files.iter().any(|f| {
        f.path.extension().map(|e| e == "unity").unwrap_or(false)
    }) || ctx.root.join("Assets").join("Scripts").is_dir()
}

pub fn has_bevy_dep(ctx: &ProjectContext) -> bool {
    cargo_toml_has_dep(ctx, "bevy")
}

pub fn has_uproject(ctx: &ProjectContext) -> bool {
    ctx.all_files.iter().any(|f| {
        f.path.extension().map(|e| e == "uproject").unwrap_or(false)
    })
}

// ── Frontend signals ────────────────────────────────────────────────

pub fn has_next_config(ctx: &ProjectContext) -> bool {
    ctx.has_file("next.config.js")
        || ctx.has_file("next.config.mjs")
        || ctx.has_file("next.config.ts")
}

pub fn has_nuxt_config(ctx: &ProjectContext) -> bool {
    ctx.has_file("nuxt.config.js")
        || ctx.has_file("nuxt.config.ts")
}

pub fn has_vite_or_vue_config(ctx: &ProjectContext) -> bool {
    ctx.has_file("vite.config.js")
        || ctx.has_file("vite.config.ts")
        || ctx.has_file("vue.config.js")
}

pub fn has_react_scripts_dep(ctx: &ProjectContext) -> bool {
    package_json_has_dep(ctx, "react-scripts")
}

// ── IaC signals ─────────────────────────────────────────────────────

pub fn has_tf_files(ctx: &ProjectContext) -> bool {
    ctx.all_files.iter().any(|f| {
        f.path.extension().map(|e| e == "tf").unwrap_or(false)
    })
}

// ── Serverless signals ──────────────────────────────────────────────

pub fn has_sam_template(ctx: &ProjectContext) -> bool {
    if let Some(content) = ctx.read_root_file("template.yaml") {
        return content.contains("AWS::Serverless");
    }
    if let Some(content) = ctx.read_root_file("template.yml") {
        return content.contains("AWS::Serverless");
    }
    false
}

// ── Doc site signals ────────────────────────────────────────────────

pub fn has_docusaurus_config(ctx: &ProjectContext) -> bool {
    ctx.has_file("docusaurus.config.js")
        || ctx.has_file("docusaurus.config.ts")
}

pub fn has_hugo_config(ctx: &ProjectContext) -> bool {
    ctx.has_file("hugo.toml") || ctx.has_file("hugo.yaml") || {
        // config.toml with baseURL is likely Hugo
        if let Some(content) = ctx.read_root_file("config.toml") {
            content.contains("baseURL")
        } else {
            false
        }
    }
}

pub fn has_vuepress(ctx: &ProjectContext) -> bool {
    ctx.root.join(".vuepress").is_dir()
}

// ── Re-exports from signals_backend ─────────────────────────────────

pub use super::signals_backend::{
    has_rails_dep, has_express_dep, has_cmd_dir,
    has_rust_web_framework, has_phoenix_dep,
    is_ml_project, is_codegen_heavy,
};

// ── Helpers ─────────────────────────────────────────────────────────

pub fn package_json_has_dep(ctx: &ProjectContext, dep: &str) -> bool {
    if let Some(pj) = &ctx.package_json {
        if let Some(deps) = pj.get("dependencies").and_then(|d| d.as_object()) {
            if deps.contains_key(dep) {
                return true;
            }
        }
        if let Some(deps) = pj.get("devDependencies").and_then(|d| d.as_object()) {
            if deps.contains_key(dep) {
                return true;
            }
        }
    }
    false
}

pub fn cargo_toml_has_dep(ctx: &ProjectContext, dep: &str) -> bool {
    if let Some(content) = ctx.read_root_file("Cargo.toml") {
        // Simple heuristic: check if the dep name appears in [dependencies] section
        return content.contains(&format!("{dep} ")) || content.contains(&format!("{dep}=")) || content.contains(&format!("\"{dep}\""));
    }
    false
}
