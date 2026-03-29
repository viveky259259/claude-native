use std::fs;

use anyhow::Result;

use crate::detection::*;
use crate::scan::ProjectContext;

/// Bootstrap a project with Claude Native configuration files.
/// Only creates files that don't already exist.
pub fn init_project(ctx: &ProjectContext) -> Result<Vec<String>> {
    let mut created = Vec::new();
    let root = &ctx.root;
    let pt = ctx.project_type.as_ref();

    if !root.join("CLAUDE.md").exists() && !root.join(".claude").join("CLAUDE.md").exists() {
        let content = generate_claude_md(ctx, pt);
        fs::write(root.join("CLAUDE.md"), content)?;
        created.push("CLAUDE.md".into());
    }

    if !root.join("AGENTS.md").exists() {
        let content = generate_agents_md(ctx, pt);
        fs::write(root.join("AGENTS.md"), content)?;
        created.push("AGENTS.md".into());
    }

    if !root.join(".claudeignore").exists() {
        let content = generate_claudeignore(ctx, pt);
        fs::write(root.join(".claudeignore"), content)?;
        created.push(".claudeignore".into());
    }

    let claude_dir = root.join(".claude");
    if !claude_dir.exists() {
        fs::create_dir_all(&claude_dir)?;
    }

    let settings_path = claude_dir.join("settings.json");
    if !settings_path.exists() {
        let content = generate_settings(ctx, pt);
        fs::write(&settings_path, content)?;
        created.push(".claude/settings.json".into());
    }

    Ok(created)
}

fn generate_agents_md(ctx: &ProjectContext, pt: Option<&ProjectType>) -> String {
    let project_name = ctx.root.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Project");
    let (build_cmd, test_cmd) = detect_commands(ctx, pt);

    format!(
        "# {project_name}\n\
         \n\
         > Universal AI agent instructions (AGENTS.md standard)\n\
         \n\
         ## Build & Test\n\
         \n\
         - Build: `{build_cmd}`\n\
         - Test: `{test_cmd}`\n\
         \n\
         ## Guidelines\n\
         \n\
         - Follow existing code patterns and conventions\n\
         - Write tests for new functionality\n\
         - Keep functions focused and under 80 lines\n\
         - Document non-obvious decisions with comments explaining \"why\"\n"
    )
}

fn generate_claude_md(ctx: &ProjectContext, pt: Option<&ProjectType>) -> String {
    let project_name = ctx.root.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Project");

    let (build_cmd, test_cmd) = detect_commands(ctx, pt);

    format!(
        "# {project_name}\n\
         \n\
         ## Build & Test\n\
         \n\
         ```\n\
         {build_cmd}\n\
         {test_cmd}\n\
         ```\n\
         \n\
         ## Architecture\n\
         \n\
         <!-- Describe your project structure, key modules, and data flow here -->\n\
         \n\
         ## Code Patterns\n\
         \n\
         <!-- Document error handling, data access, and API conventions here -->\n"
    )
}

pub fn detect_commands(_ctx: &ProjectContext, pt: Option<&ProjectType>) -> (&'static str, &'static str) {
    match pt.map(|p| &p.primary) {
        Some(PrimaryType::Mobile(MobileFramework::Flutter)) => ("flutter build", "flutter test"),
        Some(PrimaryType::Mobile(MobileFramework::ReactNative)) => ("npm run build", "npm test"),
        Some(PrimaryType::Mobile(MobileFramework::IosNative)) => ("swift build", "swift test"),
        Some(PrimaryType::Mobile(MobileFramework::AndroidNative)) => ("./gradlew assembleDebug", "./gradlew test"),
        Some(PrimaryType::Frontend(FrontendFramework::NextJs)) => ("npm run build", "npm test"),
        Some(PrimaryType::Frontend(_)) => ("npm run build", "npm test"),
        Some(PrimaryType::Backend(BackendFramework::Django)) => ("python manage.py check", "python manage.py test"),
        Some(PrimaryType::Backend(BackendFramework::Rails)) => ("rails db:migrate", "rails test"),
        Some(PrimaryType::Backend(BackendFramework::Express)) => ("npm run build", "npm test"),
        Some(PrimaryType::Backend(BackendFramework::GoService)) => ("go build ./...", "go test ./..."),
        Some(PrimaryType::Backend(BackendFramework::RustService)) => ("cargo build", "cargo test"),
        Some(PrimaryType::Backend(BackendFramework::Phoenix)) => ("mix compile", "mix test"),
        Some(PrimaryType::IaC(IaCTool::Terraform)) => ("terraform validate", "terraform test"),
        Some(PrimaryType::IaC(IaCTool::Helm)) => ("helm lint .", "helm template . | kubeval"),
        Some(PrimaryType::Serverless(_)) => ("sam build", "sam local invoke"),
        Some(PrimaryType::ML) => ("pip install -e .", "pytest"),
        Some(PrimaryType::DocSite(_)) => ("npm run build", "npm run build"),
        Some(PrimaryType::GameDev(GameEngine::Godot)) => ("godot --export", "godot --test"),
        Some(PrimaryType::GameDev(GameEngine::Bevy)) => ("cargo build", "cargo test"),
        _ => {
            // Detect from manifests
            if std::path::Path::new("Cargo.toml").exists() {
                ("cargo build", "cargo test")
            } else if std::path::Path::new("package.json").exists() {
                ("npm run build", "npm test")
            } else if std::path::Path::new("go.mod").exists() {
                ("go build ./...", "go test ./...")
            } else if std::path::Path::new("requirements.txt").exists() {
                ("pip install -e .", "pytest")
            } else {
                ("make build", "make test")
            }
        }
    }
}

fn generate_claudeignore(_ctx: &ProjectContext, pt: Option<&ProjectType>) -> String {
    let mut lines = base_ignore_patterns();
    if let Some(pt) = pt {
        lines.extend(type_specific_ignores(&pt.primary));
    }
    lines.join("\n") + "\n"
}

fn base_ignore_patterns() -> Vec<&'static str> {
    vec![
        "# Dependencies", "node_modules/", ".venv/", "venv/", "vendor/", "",
        "# Build artifacts", "dist/", "build/", "target/", "out/", "coverage/", "",
        "# Lock files", "package-lock.json", "yarn.lock", "pnpm-lock.yaml",
        "Cargo.lock", "Gemfile.lock", "poetry.lock", "go.sum", "",
        "# Secrets", ".env", ".env.*", "!.env.example", "",
        "# OS / IDE", ".DS_Store", ".vscode/", ".idea/", "",
        "# Logs", "*.log",
    ]
}

fn type_specific_ignores(primary: &PrimaryType) -> Vec<&'static str> {
    match primary {
        PrimaryType::Mobile(MobileFramework::Flutter) => vec![
            "", "# Flutter", ".dart_tool/", "ios/Pods/",
            "android/.gradle/", "android/build/", "*.g.dart", "*.freezed.dart",
        ],
        PrimaryType::Mobile(MobileFramework::ReactNative) => vec![
            "", "# React Native", "ios/Pods/", "android/.gradle/", "android/build/", ".expo/",
        ],
        PrimaryType::Frontend(FrontendFramework::NextJs) => vec!["", "# Next.js", ".next/", ".vercel/"],
        PrimaryType::IaC(IaCTool::Terraform) => vec!["", "# Terraform", ".terraform/", "*.tfstate", "*.tfstate.backup"],
        PrimaryType::ML => vec!["", "# ML", "*.pkl", "*.h5", "*.pth", "*.onnx", "*.safetensors", "data/", "*.csv", "*.parquet"],
        PrimaryType::GameDev(_) => vec!["", "# Game assets", "*.unity", "*.prefab", "*.tscn", "*.meta", "Library/", ".godot/", "*.png", "*.wav"],
        _ => vec![],
    }
}

fn generate_settings(_ctx: &ProjectContext, pt: Option<&ProjectType>) -> String {
    let (build_cmd, test_cmd) = match pt.map(|p| &p.primary) {
        Some(PrimaryType::Backend(BackendFramework::GoService)) => ("go", "go"),
        Some(PrimaryType::Backend(BackendFramework::RustService)) | None => ("cargo", "cargo"),
        Some(PrimaryType::Backend(BackendFramework::Django)) => ("python", "python"),
        Some(PrimaryType::Mobile(MobileFramework::Flutter)) => ("flutter", "flutter"),
        _ => ("npm", "npm"),
    };

    format!(
        r#"{{
  "permissions": {{
    "allow": [
      "Bash({build_cmd}:*)",
      "Bash({test_cmd}:*)",
      "Bash(git status:*)",
      "Bash(git diff:*)",
      "Bash(git log:*)"
    ]
  }}
}}
"#
    )
}
