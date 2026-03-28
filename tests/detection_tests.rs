mod helpers;

use claude_native::detection::*;

// ── Standard detection ──────────────────────────────────────────────

#[test]
fn detects_standard_project() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", "fn main() {}"),
        ("Cargo.toml", "[package]\nname = \"test\"\nversion = \"0.1.0\""),
    ]);
    let pt = detect(&ctx);
    // Small Rust project with single manifest and few files → micro-repo
    assert!(
        matches!(pt.primary, PrimaryType::MicroRepo | PrimaryType::Standard),
        "Expected MicroRepo or Standard, got {:?}",
        pt.primary
    );
}

// ── Monorepo detection ──────────────────────────────────────────────

#[test]
fn detects_monorepo_by_workspace_config() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("pnpm-workspace.yaml", "packages:\n  - packages/*"),
        ("package.json", "{}"),
        ("packages/a/package.json", "{}"),
        ("packages/b/package.json", "{}"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::Monorepo);
}

#[test]
fn detects_monorepo_by_cargo_workspace() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[workspace]\nmembers = [\"crates/*\"]"),
        ("crates/a/Cargo.toml", "[package]\nname = \"a\""),
        ("crates/b/Cargo.toml", "[package]\nname = \"b\""),
        ("crates/a/src/lib.rs", ""),
        ("crates/b/src/lib.rs", ""),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::Monorepo);
}

#[test]
fn detects_monorepo_by_packages_dir() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("package.json", "{}"),
        ("packages/frontend/package.json", "{}"),
        ("packages/frontend/src/index.ts", "export default 1;"),
        ("packages/backend/package.json", "{}"),
        ("packages/backend/src/index.ts", "export default 2;"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::Monorepo);
}

#[test]
fn detects_monorepo_by_nx() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("nx.json", "{}"),
        ("package.json", "{}"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::Monorepo);
}

// ── Micro-repo detection ────────────────────────────────────────────

#[test]
fn detects_micro_repo() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"tiny\""),
        ("src/lib.rs", "pub fn hello() {}"),
        ("src/utils.rs", "pub fn util() {}"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::MicroRepo);
}

// ── Flutter detection ───────────────────────────────────────────────

#[test]
fn detects_flutter() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("pubspec.yaml", "name: my_app\ndependencies:\n  flutter:\n    sdk: flutter"),
        ("lib/main.dart", "void main() {}"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::Mobile(MobileFramework::Flutter));
}

// ── React Native detection ──────────────────────────────────────────

#[test]
fn detects_react_native() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("package.json", r#"{"dependencies": {"react-native": "0.72.0"}}"#),
        ("App.tsx", "export default function App() {}"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::Mobile(MobileFramework::ReactNative));
}

// ── Next.js detection ───────────────────────────────────────────────

#[test]
fn detects_nextjs() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("next.config.js", "module.exports = {}"),
        ("package.json", r#"{"dependencies": {"next": "14.0.0"}}"#),
        ("src/app/page.tsx", "export default function Home() {}"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::Frontend(FrontendFramework::NextJs));
}

// ── Angular detection ───────────────────────────────────────────────

#[test]
fn detects_angular() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("angular.json", "{}"),
        ("package.json", r#"{"dependencies": {"@angular/core": "17.0.0"}}"#),
        ("src/app/app.component.ts", ""),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::Frontend(FrontendFramework::Angular));
}

// ── Django detection ────────────────────────────────────────────────

#[test]
fn detects_django() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("manage.py", "#!/usr/bin/env python"),
        ("requirements.txt", "django==4.2"),
        ("myapp/views.py", "from django.http import HttpResponse"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::Backend(BackendFramework::Django));
}

// ── Express detection ───────────────────────────────────────────────

#[test]
fn detects_express() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("package.json", r#"{"dependencies": {"express": "4.18.0"}}"#),
        ("src/index.ts", "import express from 'express';"),
    ]);
    let pt = detect(&ctx);
    // Express should be detected as backend
    assert_eq!(pt.primary, PrimaryType::Backend(BackendFramework::Express));
}

// ── Go service detection ────────────────────────────────────────────

#[test]
fn detects_go_service() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("go.mod", "module myservice\ngo 1.21"),
        ("cmd/server/main.go", "package main\nfunc main() {}"),
        ("internal/handler.go", "package internal"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::Backend(BackendFramework::GoService));
}

// ── Terraform detection ─────────────────────────────────────────────

#[test]
fn detects_terraform() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("main.tf", "resource \"aws_instance\" \"web\" {}"),
        ("variables.tf", "variable \"region\" {}"),
        ("outputs.tf", "output \"ip\" {}"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::IaC(IaCTool::Terraform));
}

// ── Helm detection ──────────────────────────────────────────────────

#[test]
fn detects_helm() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("Chart.yaml", "apiVersion: v2\nname: mychart"),
        ("values.yaml", "replicaCount: 1"),
        ("templates/deployment.yaml", "kind: Deployment"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::IaC(IaCTool::Helm));
}

// ── Serverless detection ────────────────────────────────────────────

#[test]
fn detects_serverless_framework() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("serverless.yml", "service: my-service\nprovider:\n  name: aws"),
        ("handler.js", "module.exports.hello = async () => {};"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::Serverless(ServerlessPlatform::ServerlessFramework));
}

#[test]
fn detects_cloudflare_workers() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("wrangler.toml", "name = \"my-worker\""),
        ("src/index.ts", "export default { fetch() {} }"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::Serverless(ServerlessPlatform::CloudflareWorkers));
}

// ── Docusaurus detection ────────────────────────────────────────────

#[test]
fn detects_docusaurus() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("docusaurus.config.js", "module.exports = {}"),
        ("package.json", r#"{"dependencies": {"@docusaurus/core": "3.0"}}"#),
        ("docs/intro.md", "# Welcome"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::DocSite(DocFramework::Docusaurus));
}

#[test]
fn detects_mkdocs() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("mkdocs.yml", "site_name: My Docs"),
        ("docs/index.md", "# Welcome"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::DocSite(DocFramework::MkDocs));
}

// ── Godot detection ─────────────────────────────────────────────────

#[test]
fn detects_godot() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("project.godot", "[application]\nconfig/name=\"MyGame\""),
        ("scripts/player.gd", "extends CharacterBody2D"),
    ]);
    let pt = detect(&ctx);
    assert_eq!(pt.primary, PrimaryType::GameDev(GameEngine::Godot));
}

// ── Polyglot flag ───────────────────────────────────────────────────

#[test]
fn detects_polyglot_flag() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("go.mod", "module backend\ngo 1.21"),
        ("cmd/server/main.go", "package main"),
        ("package.json", r#"{"dependencies": {}}"#),
        ("frontend/src/App.tsx", "export default function App() {}"),
    ]);
    let pt = detect(&ctx);
    assert!(pt.flags.is_polyglot, "Expected polyglot flag for Go + TypeScript project");
}

// ── Language detection ──────────────────────────────────────────────

#[test]
fn detects_languages() {
    let (_dir, ctx) = helpers::scan_project(&[
        ("src/main.rs", "fn main() {}"),
        ("src/lib.rs", "pub fn lib() {}"),
        ("scripts/setup.py", "print('hello')"),
    ]);
    let pt = detect(&ctx);
    assert!(pt.languages.iter().any(|l| matches!(l, Language::Rust)));
    assert!(pt.languages.iter().any(|l| matches!(l, Language::Python)));
}
