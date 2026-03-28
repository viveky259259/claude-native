use std::collections::HashMap;

use crate::detection::{PrimaryType, ProjectType};
use crate::rules::Dimension;

/// Get dimension weights for a project type (from GOLDEN_RULES.md section 6.15)
pub fn get_weights(project_type: &ProjectType) -> HashMap<Dimension, f64> {
    let base = match &project_type.primary {
        PrimaryType::Standard => standard_weights(),
        PrimaryType::Monorepo => monorepo_weights(),
        PrimaryType::MicroRepo => micro_repo_weights(),
        PrimaryType::Mobile(_) => mobile_weights(),
        PrimaryType::Frontend(_) => frontend_weights(),
        PrimaryType::Backend(_) => backend_weights(),
        PrimaryType::IaC(_) => iac_weights(),
        PrimaryType::Serverless(_) => serverless_weights(),
        PrimaryType::ML => ml_weights(),
        PrimaryType::CodegenHeavy => codegen_weights(),
        PrimaryType::DocSite(_) => doc_site_weights(),
        PrimaryType::GameDev(_) => game_dev_weights(),
    };

    // Apply flag adjustments
    let mut weights = base;
    if project_type.flags.is_legacy {
        // Legacy: boost Foundation and Code Quality
        adjust(&mut weights, Dimension::Foundation, 0.05);
        adjust(&mut weights, Dimension::CodeQuality, 0.05);
        adjust(&mut weights, Dimension::ContextEfficiency, -0.05);
        adjust(&mut weights, Dimension::Tooling, -0.05);
    }

    weights
}

fn adjust(weights: &mut HashMap<Dimension, f64>, dim: Dimension, delta: f64) {
    if let Some(w) = weights.get_mut(&dim) {
        *w = (*w + delta).max(0.05).min(0.50);
    }
}

fn standard_weights() -> HashMap<Dimension, f64> {
    HashMap::from([
        (Dimension::Foundation, 0.25),
        (Dimension::ContextEfficiency, 0.25),
        (Dimension::Navigation, 0.20),
        (Dimension::Tooling, 0.15),
        (Dimension::CodeQuality, 0.15),
    ])
}

fn monorepo_weights() -> HashMap<Dimension, f64> {
    HashMap::from([
        (Dimension::Foundation, 0.30),
        (Dimension::ContextEfficiency, 0.25),
        (Dimension::Navigation, 0.20),
        (Dimension::Tooling, 0.15),
        (Dimension::CodeQuality, 0.10),
    ])
}

fn micro_repo_weights() -> HashMap<Dimension, f64> {
    HashMap::from([
        (Dimension::Foundation, 0.20),
        (Dimension::ContextEfficiency, 0.20),
        (Dimension::Navigation, 0.15),
        (Dimension::Tooling, 0.15),
        (Dimension::CodeQuality, 0.30),
    ])
}

fn mobile_weights() -> HashMap<Dimension, f64> {
    HashMap::from([
        (Dimension::Foundation, 0.25),
        (Dimension::ContextEfficiency, 0.30),
        (Dimension::Navigation, 0.15),
        (Dimension::Tooling, 0.15),
        (Dimension::CodeQuality, 0.15),
    ])
}

fn frontend_weights() -> HashMap<Dimension, f64> {
    HashMap::from([
        (Dimension::Foundation, 0.25),
        (Dimension::ContextEfficiency, 0.30),
        (Dimension::Navigation, 0.15),
        (Dimension::Tooling, 0.15),
        (Dimension::CodeQuality, 0.15),
    ])
}

fn backend_weights() -> HashMap<Dimension, f64> {
    HashMap::from([
        (Dimension::Foundation, 0.25),
        (Dimension::ContextEfficiency, 0.20),
        (Dimension::Navigation, 0.20),
        (Dimension::Tooling, 0.15),
        (Dimension::CodeQuality, 0.20),
    ])
}

fn iac_weights() -> HashMap<Dimension, f64> {
    HashMap::from([
        (Dimension::Foundation, 0.30),
        (Dimension::ContextEfficiency, 0.20),
        (Dimension::Navigation, 0.15),
        (Dimension::Tooling, 0.20),
        (Dimension::CodeQuality, 0.15),
    ])
}

fn serverless_weights() -> HashMap<Dimension, f64> {
    HashMap::from([
        (Dimension::Foundation, 0.25),
        (Dimension::ContextEfficiency, 0.25),
        (Dimension::Navigation, 0.15),
        (Dimension::Tooling, 0.20),
        (Dimension::CodeQuality, 0.15),
    ])
}

fn ml_weights() -> HashMap<Dimension, f64> {
    HashMap::from([
        (Dimension::Foundation, 0.20),
        (Dimension::ContextEfficiency, 0.35),
        (Dimension::Navigation, 0.10),
        (Dimension::Tooling, 0.15),
        (Dimension::CodeQuality, 0.20),
    ])
}

fn codegen_weights() -> HashMap<Dimension, f64> {
    HashMap::from([
        (Dimension::Foundation, 0.25),
        (Dimension::ContextEfficiency, 0.30),
        (Dimension::Navigation, 0.15),
        (Dimension::Tooling, 0.20),
        (Dimension::CodeQuality, 0.10),
    ])
}

fn doc_site_weights() -> HashMap<Dimension, f64> {
    HashMap::from([
        (Dimension::Foundation, 0.20),
        (Dimension::ContextEfficiency, 0.25),
        (Dimension::Navigation, 0.20),
        (Dimension::Tooling, 0.15),
        (Dimension::CodeQuality, 0.20),
    ])
}

fn game_dev_weights() -> HashMap<Dimension, f64> {
    HashMap::from([
        (Dimension::Foundation, 0.25),
        (Dimension::ContextEfficiency, 0.35),
        (Dimension::Navigation, 0.15),
        (Dimension::Tooling, 0.10),
        (Dimension::CodeQuality, 0.15),
    ])
}
