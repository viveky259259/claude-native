use std::path::Path;

use crate::scan::ManifestKind;

// ── Skip patterns (hardcoded sensible defaults) ─────────────────────

const SKIP_DIRS: &[&str] = &[
    ".git", "node_modules", "target", ".next", "dist", "build",
    "__pycache__", ".venv", "venv", "vendor", ".dart_tool",
    ".gradle", "Pods", "DerivedData", ".build", ".expo",
    ".aws-sam", ".serverless", ".terraform", "Library",
    ".godot", ".import", "coverage", ".turbo", ".nuxt",
    ".angular", ".cache",
];

pub fn should_skip_dir(name: &str) -> bool {
    SKIP_DIRS.contains(&name)
}

pub fn is_source_extension(ext: &str) -> bool {
    matches!(ext,
        "rs" | "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" |
        "py" | "go" | "dart" | "swift" | "kt" | "kts" | "java" |
        "rb" | "cs" | "ex" | "exs" | "cpp" | "cc" | "c" | "h" | "hpp" |
        "vue" | "svelte" | "tf" | "hcl" | "yaml" | "yml" | "toml" | "json" |
        "graphql" | "proto" | "prisma" | "sql" | "sh" | "bash" | "zsh" |
        "md" | "mdx" | "css" | "scss" | "less" | "html" | "xml"
    )
}

pub fn is_test_file(path: &Path) -> bool {
    let name = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    let full = path.to_string_lossy();

    name.ends_with("_test")
        || name.ends_with(".test")
        || name.ends_with("_spec")
        || name.ends_with(".spec")
        || name.starts_with("test_")
        || full.contains("__tests__")
        || full.contains("/tests/")
        || full.contains("/test/")
        || full.contains("/spec/")
}

pub fn is_generated_file(path: &Path) -> bool {
    let full = path.to_string_lossy();
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    full.contains("/generated/")
        || full.contains("/gen/")
        || name.contains(".g.")
        || name.contains(".generated.")
        || name.contains(".freezed.")
        || name.contains(".pb.")
        || name == "R.java"
        || name == "R.kt"
        || name == "BuildConfig.kt"
        || name == "BuildConfig.java"
}

pub fn is_lock_file(name: &str) -> bool {
    matches!(name,
        "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml" |
        "Cargo.lock" | "Gemfile.lock" | "poetry.lock" | "go.sum" |
        "composer.lock" | "pubspec.lock" | "Pipfile.lock"
    )
}

pub fn is_ci_config(path: &Path) -> bool {
    let full = path.to_string_lossy();
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    full.contains(".github/workflows")
        || name == ".gitlab-ci.yml"
        || name == "Jenkinsfile"
        || name == ".circleci"
        || name == ".travis.yml"
        || name == "azure-pipelines.yml"
        || name == "bitbucket-pipelines.yml"
}

pub fn is_env_file(name: &str) -> bool {
    name == ".env"
        || name.starts_with(".env.")
        || name.ends_with(".env")
}

pub fn is_manifest(name: &str) -> Option<ManifestKind> {
    match name {
        "package.json" => Some(ManifestKind::PackageJson),
        "Cargo.toml" => Some(ManifestKind::CargoToml),
        "go.mod" => Some(ManifestKind::GoMod),
        "pubspec.yaml" => Some(ManifestKind::PubspecYaml),
        "requirements.txt" => Some(ManifestKind::RequirementsTxt),
        "pyproject.toml" => Some(ManifestKind::PyprojectToml),
        "Gemfile" => Some(ManifestKind::Gemfile),
        "mix.exs" => Some(ManifestKind::MixExs),
        _ => None,
    }
}
