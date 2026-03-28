mod helpers;

use claude_native::rules::Rule;
use claude_native::rules::project_specific::*;

// ═══════════════════════════════════════════════════════════════════
// Monorepo rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn monorepo_per_package_test_commands_pass_with_subdirs() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Mono\nTest backend: `cd packages/api && npm test`\nBuild: `npm run build`"),
        ("pnpm-workspace.yaml", "packages:\n  - packages/*"),
        ("packages/api/package.json", "{}"),
        ("packages/web/package.json", "{}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = monorepo::PerPackageTestCommands.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn monorepo_shared_tooling_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("pnpm-workspace.yaml", "packages:\n  - packages/*"),
        ("tsconfig.json", "{}"),
        ("packages/a/package.json", "{}"),
        ("packages/b/package.json", "{}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = monorepo::SharedToolingConfig.check(&ctx);
    assert!(result.status.is_pass());
}

// ═══════════════════════════════════════════════════════════════════
// Micro-repo rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn micro_repo_comprehensive_tests_pass_with_good_ratio() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"lib\""),
        ("src/lib.rs", "pub fn a() {}"),
        ("src/utils.rs", "pub fn b() {}"),
        ("tests/lib_test.rs", "#[test] fn t() {}"),
        ("tests/utils_test.rs", "#[test] fn t() {}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = micro_repo::ComprehensiveTests.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn micro_repo_examples_pass_with_readme_code() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("Cargo.toml", "[package]\nname = \"lib\""),
        ("src/lib.rs", "pub fn a() {}"),
        ("README.md", "# Usage\n\n```rust\nuse lib::a;\na();\n```\n\nMore example code here."),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = micro_repo::ExamplesExist.check(&ctx);
    assert!(result.status.is_pass());
}

// ═══════════════════════════════════════════════════════════════════
// Mobile rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn mobile_platform_commands_pass_flutter() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("pubspec.yaml", "name: app"),
        ("CLAUDE.md", "# App\nAnalyze: `flutter analyze`\nTest: `flutter test`"),
        ("lib/main.dart", "void main() {}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = mobile::PlatformCommands.check(&ctx);
    assert!(result.status.is_pass());
}

// ═══════════════════════════════════════════════════════════════════
// Frontend rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn frontend_env_example_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("next.config.js", "module.exports = {}"),
        ("package.json", r#"{"dependencies":{"next":"14"}}"#),
        (".env", "SECRET=x"),
        (".env.example", "SECRET="),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = frontend::EnvExampleExists.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn frontend_env_example_fails_without() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("next.config.js", "module.exports = {}"),
        ("package.json", r#"{"dependencies":{"next":"14"}}"#),
        (".env", "SECRET=x"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = frontend::EnvExampleExists.check(&ctx);
    assert!(result.status.is_failure());
}

#[test]
fn frontend_typecheck_before_build_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("next.config.js", "module.exports = {}"),
        ("package.json", r#"{"dependencies":{"next":"14"}}"#),
        ("CLAUDE.md", "# App\nType check: `tsc --noEmit`\nBuild: `npm run build`\nTest: `npm test`"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = frontend::TypeCheckBeforeBuild.check(&ctx);
    assert!(result.status.is_pass());
}

// ═══════════════════════════════════════════════════════════════════
// Backend rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn backend_log_files_ignored_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("manage.py", "#!/usr/bin/env python"),
        ("requirements.txt", "django==4.2"),
        (".claudeignore", "*.log\nlogs/\n"),
        ("logs/app.log", "some log"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = backend::LogFilesIgnored.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn backend_data_access_documented_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("manage.py", "#!/usr/bin/env python"),
        ("requirements.txt", "django==4.2"),
        ("CLAUDE.md", "# Backend\nBuild: `python manage.py build`\nTest: `python manage.py test`\n## Database\nUse Django ORM. Repository pattern in services/."),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = backend::DataAccessDocumented.check(&ctx);
    assert!(result.status.is_pass());
}

// ═══════════════════════════════════════════════════════════════════
// IaC rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn iac_state_files_blocked_passes_with_ignore() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("main.tf", "resource \"aws_instance\" \"web\" {}"),
        (".claudeignore", "*.tfstate\n.terraform/\n"),
        ("terraform.tfstate", "{}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = iac::StateFilesBlocked.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn iac_state_files_blocked_fails_without_ignore() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("main.tf", "resource \"aws_instance\" \"web\" {}"),
        ("terraform.tfstate", "{}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = iac::StateFilesBlocked.check(&ctx);
    assert!(result.status.is_failure());
}

#[test]
fn iac_module_conventions_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("main.tf", "resource \"aws_instance\" \"web\" {}"),
        ("variables.tf", "variable \"region\" {}"),
        ("outputs.tf", "output \"ip\" {}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = iac::ModuleConventions.check(&ctx);
    assert!(result.status.is_pass());
}

// ═══════════════════════════════════════════════════════════════════
// Serverless rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn serverless_test_events_pass() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("serverless.yml", "service: my-svc"),
        ("handler.js", "module.exports.hello = async () => {};"),
        ("events/api-event.json", "{}"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = serverless::TestEventsExist.check(&ctx);
    assert!(result.status.is_pass());
}

// ═══════════════════════════════════════════════════════════════════
// ML rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn ml_notebooks_not_primary_passes_with_more_py() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("requirements.txt", "torch==2.1.0"),
        ("notebook.ipynb", "{}"),
        ("src/train.py", "import torch"),
        ("src/model.py", "class Model: pass"),
        ("src/data.py", "def load(): pass"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = ml::NotebooksNotPrimary.check(&ctx);
    assert!(result.status.is_pass());
}

// ═══════════════════════════════════════════════════════════════════
// Codegen rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn codegen_edit_specs_passes_with_instruction() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("api.proto", "syntax = \"proto3\";"),
        ("generated/api.pb.go", "// generated"),
        ("CLAUDE.md", "# API\nBuild: `make`\nTest: `make test`\nDo not edit generated code. Edit .proto files instead."),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = codegen::EditSpecsNotGenerated.check(&ctx);
    assert!(result.status.is_pass());
}

#[test]
fn codegen_regen_command_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("api.proto", "syntax = \"proto3\";"),
        ("generated/api.pb.go", "// generated"),
        ("CLAUDE.md", "# API\nBuild: `make`\nTest: `make test`\nRegenerate: `protoc --go_out=. api.proto`"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = codegen::RegenCommandDocumented.check(&ctx);
    assert!(result.status.is_pass());
}

// ═══════════════════════════════════════════════════════════════════
// Legacy rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn legacy_documents_mess_passes() {
    let big = "x = 1\n".repeat(400);
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("CLAUDE.md", "# Legacy\nBuild: `make`\nTest: `make test`\n## Known Issues\nDo not use the legacy pattern in old_auth/."),
        ("src/a.py", &big), ("src/b.py", &big), ("src/c.py", &big),
        ("src/d.py", &big), ("src/e.py", &big), ("src/f.py", &big),
        ("src/g.py", &big), ("src/h.py", &big), ("src/i.py", &big),
        ("src/j.py", &big), ("src/k.py", &big), ("src/l.py", &big),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    // Force legacy flag since detect might not flag it (has CLAUDE.md)
    let mut pt = pt;
    pt.flags.is_legacy = true;
    ctx.project_type = Some(pt);
    let result = legacy::DocumentsTheMess.check(&ctx);
    assert!(result.status.is_pass());
}

// ═══════════════════════════════════════════════════════════════════
// Doc site rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn doc_site_build_output_ignored_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("docusaurus.config.js", "module.exports = {}"),
        ("package.json", r#"{"dependencies":{"@docusaurus/core":"3.0"}}"#),
        (".claudeignore", "build/\n.docusaurus/\nnode_modules/\n"),
        ("docs/intro.md", "# Welcome"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = doc_site::BuildOutputIgnored.check(&ctx);
    assert!(result.status.is_pass());
}

// ═══════════════════════════════════════════════════════════════════
// Game dev rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn game_dev_scripts_primary_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("project.godot", "[application]"),
        ("CLAUDE.md", "# Game\nBuild: `godot --export`\nTest: `godot --test`\nScripts: scripts/"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = game_dev::ScriptsArePrimary.check(&ctx);
    assert!(result.status.is_pass());
}

// ═══════════════════════════════════════════════════════════════════
// Polyglot rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn polyglot_independent_build_test_passes() {
    let (_dir, mut ctx) = helpers::scan_project(&[
        ("go.mod", "module backend\ngo 1.21"),
        ("cmd/server/main.go", "package main"),
        ("package.json", r#"{"dependencies":{}}"#),
        ("frontend/src/App.tsx", "export default function App() {}"),
        ("CLAUDE.md", "# Polyglot\nBackend build: `cd backend && go build`\nBackend test: `cd backend && go test ./...`\nFrontend test: `cd frontend && npm test`"),
    ]);
    let pt = claude_native::detection::detect(&ctx);
    ctx.project_type = Some(pt);
    let result = polyglot::IndependentBuildTest.check(&ctx);
    assert!(result.status.is_pass());
}
