use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::detection::ProjectType;
use crate::init;
use crate::scan::ProjectContext;

/// Generate configs for multiple AI tools alongside CLAUDE.md.
pub fn generate_all(ctx: &ProjectContext) -> Result<Vec<String>> {
    let mut created = Vec::new();
    let root = &ctx.root;
    let pt = ctx.project_type.as_ref();

    // Standard --init files
    let init_files = init::init_project(ctx)?;
    created.extend(init_files);

    // .cursorrules (Cursor IDE)
    if !root.join(".cursorrules").exists() {
        fs::write(root.join(".cursorrules"), generate_cursorrules(ctx, pt))?;
        created.push(".cursorrules".into());
    }

    // .github/copilot-instructions.md (GitHub Copilot)
    let copilot_dir = root.join(".github");
    let copilot_path = copilot_dir.join("copilot-instructions.md");
    if !copilot_path.exists() {
        fs::create_dir_all(&copilot_dir)?;
        fs::write(&copilot_path, generate_copilot_instructions(ctx, pt))?;
        created.push(".github/copilot-instructions.md".into());
    }

    Ok(created)
}

fn generate_cursorrules(ctx: &ProjectContext, pt: Option<&ProjectType>) -> String {
    let name = project_name(ctx);
    let (build, test) = init::detect_commands(ctx, pt);
    format!(
        "# {name} — Cursor Rules\n\
         \n\
         ## Build & Test\n\
         - Build: `{build}`\n\
         - Test: `{test}`\n\
         \n\
         ## Code Style\n\
         - Follow existing patterns in the codebase\n\
         - Write tests for new functionality\n\
         - Keep functions focused and concise\n\
         - Use descriptive variable and function names\n"
    )
}

fn generate_copilot_instructions(ctx: &ProjectContext, pt: Option<&ProjectType>) -> String {
    let name = project_name(ctx);
    let (build, test) = init::detect_commands(ctx, pt);
    format!(
        "# {name} — Copilot Instructions\n\
         \n\
         ## Project\n\
         Build: `{build}`\n\
         Test: `{test}`\n\
         \n\
         ## Guidelines\n\
         - Follow existing code patterns and conventions\n\
         - Write tests for all new functionality\n\
         - Keep functions under 80 lines\n\
         - Prefer explicit types over inference where it aids readability\n"
    )
}

fn project_name(ctx: &ProjectContext) -> String {
    ctx.root.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Project")
        .to_string()
}
