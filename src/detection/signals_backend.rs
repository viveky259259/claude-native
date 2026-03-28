use crate::scan::ProjectContext;

// ── Backend signals ─────────────────────────────────────────────────

pub fn has_rails_dep(ctx: &ProjectContext) -> bool {
    if let Some(content) = ctx.read_root_file("Gemfile") {
        return content.contains("rails");
    }
    false
}

pub fn has_express_dep(ctx: &ProjectContext) -> bool {
    super::signals::package_json_has_dep(ctx, "express")
}

pub fn has_cmd_dir(ctx: &ProjectContext) -> bool {
    ctx.root.join("cmd").is_dir()
}

pub fn has_rust_web_framework(ctx: &ProjectContext) -> bool {
    super::signals::cargo_toml_has_dep(ctx, "actix")
        || super::signals::cargo_toml_has_dep(ctx, "axum")
        || super::signals::cargo_toml_has_dep(ctx, "rocket")
}

pub fn has_phoenix_dep(ctx: &ProjectContext) -> bool {
    if let Some(content) = ctx.read_root_file("mix.exs") {
        return content.contains("phoenix");
    }
    false
}

// ── ML signals ──────────────────────────────────────────────────────

pub fn is_ml_project(ctx: &ProjectContext) -> bool {
    let has_notebooks = ctx.all_files.iter().any(|f| {
        f.path.extension().map(|e| e == "ipynb").unwrap_or(false)
    });

    let has_ml_deps = if let Some(content) = ctx.read_root_file("requirements.txt") {
        content.contains("torch")
            || content.contains("tensorflow")
            || content.contains("sklearn")
            || content.contains("scikit-learn")
            || content.contains("keras")
            || content.contains("jax")
    } else if let Some(content) = ctx.read_root_file("pyproject.toml") {
        content.contains("torch")
            || content.contains("tensorflow")
            || content.contains("scikit-learn")
    } else {
        false
    };

    has_notebooks && has_ml_deps
}

// ── Codegen signals ─────────────────────────────────────────────────

pub fn is_codegen_heavy(ctx: &ProjectContext) -> bool {
    let has_proto = ctx.all_files.iter().any(|f| {
        f.path.extension().map(|e| e == "proto").unwrap_or(false)
    });
    let has_graphql = ctx.has_file("schema.graphql")
        || ctx.all_files.iter().any(|f| {
            f.path.extension().map(|e| e == "graphql").unwrap_or(false)
        });
    let has_prisma = ctx.has_file("prisma/schema.prisma") || ctx.has_file("schema.prisma");
    let has_openapi = ctx.has_file("openapi.yaml")
        || ctx.has_file("openapi.yml")
        || ctx.has_file("swagger.json")
        || ctx.has_file("swagger.yaml");
    let has_buf = ctx.has_file("buf.yaml");

    [has_proto, has_graphql, has_prisma, has_openapi, has_buf]
        .iter()
        .filter(|&&x| x)
        .count() >= 1
        && ctx.all_files.iter().any(|f| {
            let p = f.relative_path.to_string_lossy();
            p.contains("generated") || p.contains("/gen/") || p.contains("_generated")
        })
}
