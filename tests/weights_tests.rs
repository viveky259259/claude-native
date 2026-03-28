use claude_native::detection::*;
use claude_native::scoring::weights;
use claude_native::rules::Dimension;

fn make_type(primary: PrimaryType) -> ProjectType {
    ProjectType {
        primary,
        flags: ProjectFlags::default(),
        languages: vec![],
    }
}

#[test]
fn mobile_weights_sum_to_100() {
    let pt = make_type(PrimaryType::Mobile(MobileFramework::Flutter));
    let w = weights::get_weights(&pt);
    let sum: f64 = w.values().sum();
    assert!((sum - 1.0).abs() < 0.001, "Mobile weights sum to {sum}");
}

#[test]
fn frontend_weights_sum_to_100() {
    let pt = make_type(PrimaryType::Frontend(FrontendFramework::NextJs));
    let w = weights::get_weights(&pt);
    let sum: f64 = w.values().sum();
    assert!((sum - 1.0).abs() < 0.001);
}

#[test]
fn backend_weights_sum_to_100() {
    let pt = make_type(PrimaryType::Backend(BackendFramework::Django));
    let w = weights::get_weights(&pt);
    let sum: f64 = w.values().sum();
    assert!((sum - 1.0).abs() < 0.001);
}

#[test]
fn iac_weights_sum_to_100() {
    let pt = make_type(PrimaryType::IaC(IaCTool::Terraform));
    let w = weights::get_weights(&pt);
    let sum: f64 = w.values().sum();
    assert!((sum - 1.0).abs() < 0.001);
}

#[test]
fn ml_weights_sum_to_100() {
    let pt = make_type(PrimaryType::ML);
    let w = weights::get_weights(&pt);
    let sum: f64 = w.values().sum();
    assert!((sum - 1.0).abs() < 0.001);
}

#[test]
fn game_dev_weights_sum_to_100() {
    let pt = make_type(PrimaryType::GameDev(GameEngine::Unity));
    let w = weights::get_weights(&pt);
    let sum: f64 = w.values().sum();
    assert!((sum - 1.0).abs() < 0.001);
}

#[test]
fn legacy_flag_adjusts_weights() {
    let pt = ProjectType {
        primary: PrimaryType::Standard,
        flags: ProjectFlags { is_polyglot: false, is_legacy: true },
        languages: vec![],
    };
    let w = weights::get_weights(&pt);
    let foundation = w[&Dimension::Foundation];
    assert!(foundation > 0.25, "Legacy should boost Foundation weight, got {foundation}");
}

#[test]
fn micro_repo_boosts_code_quality_weight() {
    let pt = make_type(PrimaryType::MicroRepo);
    let w = weights::get_weights(&pt);
    let quality = w[&Dimension::CodeQuality];
    assert_eq!(quality, 0.30, "Micro-repo should have 30% CodeQuality weight");
}
