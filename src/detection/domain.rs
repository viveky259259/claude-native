use crate::detection::*;
use crate::detection::signals;
use crate::scan::ProjectContext;

/// Detect the primary project type from domain-specific markers.
pub fn detect_domain(ctx: &ProjectContext) -> Option<PrimaryType> {
    detect_mobile(ctx)
        .or_else(|| detect_game(ctx))
        .or_else(|| detect_frontend(ctx))
        .or_else(|| detect_iac(ctx))
        .or_else(|| detect_serverless(ctx))
        .or_else(|| detect_doc_site(ctx))
        .or_else(|| detect_ml_or_codegen(ctx))
        .or_else(|| detect_backend(ctx))
}

fn detect_mobile(ctx: &ProjectContext) -> Option<PrimaryType> {
    if ctx.has_file("pubspec.yaml") {
        return Some(PrimaryType::Mobile(MobileFramework::Flutter));
    }
    if signals::has_react_native_dep(ctx) {
        return Some(PrimaryType::Mobile(MobileFramework::ReactNative));
    }
    let no_cross = !ctx.has_file("pubspec.yaml") && !signals::has_react_native_dep(ctx);
    if signals::has_xcodeproj(ctx) && no_cross {
        return Some(PrimaryType::Mobile(MobileFramework::IosNative));
    }
    if signals::has_android_gradle(ctx) && no_cross {
        return Some(PrimaryType::Mobile(MobileFramework::AndroidNative));
    }
    None
}

fn detect_game(ctx: &ProjectContext) -> Option<PrimaryType> {
    if signals::has_unity(ctx) { return Some(PrimaryType::GameDev(GameEngine::Unity)); }
    if ctx.has_file("project.godot") { return Some(PrimaryType::GameDev(GameEngine::Godot)); }
    if signals::has_bevy_dep(ctx) { return Some(PrimaryType::GameDev(GameEngine::Bevy)); }
    if signals::has_uproject(ctx) { return Some(PrimaryType::GameDev(GameEngine::Unreal)); }
    None
}

fn detect_frontend(ctx: &ProjectContext) -> Option<PrimaryType> {
    if signals::has_next_config(ctx) { return Some(PrimaryType::Frontend(FrontendFramework::NextJs)); }
    if signals::has_nuxt_config(ctx) { return Some(PrimaryType::Frontend(FrontendFramework::Nuxt)); }
    if ctx.has_file("angular.json") { return Some(PrimaryType::Frontend(FrontendFramework::Angular)); }
    if signals::has_vite_or_vue_config(ctx) { return Some(PrimaryType::Frontend(FrontendFramework::VueVite)); }
    if signals::has_react_scripts_dep(ctx) { return Some(PrimaryType::Frontend(FrontendFramework::CreateReactApp)); }
    None
}

fn detect_iac(ctx: &ProjectContext) -> Option<PrimaryType> {
    if signals::has_tf_files(ctx) { return Some(PrimaryType::IaC(IaCTool::Terraform)); }
    if ctx.has_file("Chart.yaml") { return Some(PrimaryType::IaC(IaCTool::Helm)); }
    if ctx.has_file("kustomization.yaml") { return Some(PrimaryType::IaC(IaCTool::Kustomize)); }
    if ctx.has_file("Pulumi.yaml") { return Some(PrimaryType::IaC(IaCTool::Pulumi)); }
    if ctx.has_file("cdk.json") { return Some(PrimaryType::IaC(IaCTool::AwsCdk)); }
    None
}

fn detect_serverless(ctx: &ProjectContext) -> Option<PrimaryType> {
    if ctx.has_file("serverless.yml") || ctx.has_file("serverless.yaml") {
        return Some(PrimaryType::Serverless(ServerlessPlatform::ServerlessFramework));
    }
    if signals::has_sam_template(ctx) { return Some(PrimaryType::Serverless(ServerlessPlatform::AwsSam)); }
    if ctx.has_file("wrangler.toml") { return Some(PrimaryType::Serverless(ServerlessPlatform::CloudflareWorkers)); }
    None
}

fn detect_doc_site(ctx: &ProjectContext) -> Option<PrimaryType> {
    if signals::has_docusaurus_config(ctx) { return Some(PrimaryType::DocSite(DocFramework::Docusaurus)); }
    if ctx.has_file("mkdocs.yml") { return Some(PrimaryType::DocSite(DocFramework::MkDocs)); }
    if signals::has_hugo_config(ctx) { return Some(PrimaryType::DocSite(DocFramework::Hugo)); }
    if signals::has_vuepress(ctx) { return Some(PrimaryType::DocSite(DocFramework::VuePress)); }
    None
}

fn detect_ml_or_codegen(ctx: &ProjectContext) -> Option<PrimaryType> {
    if signals::is_ml_project(ctx) { return Some(PrimaryType::ML); }
    if signals::is_codegen_heavy(ctx) { return Some(PrimaryType::CodegenHeavy); }
    None
}

fn detect_backend(ctx: &ProjectContext) -> Option<PrimaryType> {
    if ctx.has_file("manage.py") { return Some(PrimaryType::Backend(BackendFramework::Django)); }
    if signals::has_rails_dep(ctx) { return Some(PrimaryType::Backend(BackendFramework::Rails)); }
    if signals::has_express_dep(ctx) { return Some(PrimaryType::Backend(BackendFramework::Express)); }
    if ctx.has_file("go.mod") && signals::has_cmd_dir(ctx) { return Some(PrimaryType::Backend(BackendFramework::GoService)); }
    if signals::has_rust_web_framework(ctx) { return Some(PrimaryType::Backend(BackendFramework::RustService)); }
    if ctx.has_file("mix.exs") && signals::has_phoenix_dep(ctx) { return Some(PrimaryType::Backend(BackendFramework::Phoenix)); }
    None
}
