use claude_native::detection::*;
use claude_native::rules::*;
use claude_native::scoring::*;

fn make_project_type() -> ProjectType {
    ProjectType {
        primary: PrimaryType::Standard,
        flags: ProjectFlags::default(),
        languages: vec![Language::Rust],
    }
}

fn make_result(dim: Dimension, severity: Severity, status: RuleStatus) -> RuleResult {
    RuleResult {
        rule_id: "test".into(),
        name: "test rule".into(),
        dimension: dim,
        severity,
        status,
        suggestion: None,
    }
}

// ── Grade assignment ────────────────────────────────────────────────

#[test]
fn grade_a_plus_for_90_plus() {
    assert_eq!(Grade::from_score(95.0), Grade::APlus);
    assert_eq!(Grade::from_score(90.0), Grade::APlus);
}

#[test]
fn grade_a_for_80_to_89() {
    assert_eq!(Grade::from_score(85.0), Grade::A);
    assert_eq!(Grade::from_score(80.0), Grade::A);
}

#[test]
fn grade_b_for_70_to_79() {
    assert_eq!(Grade::from_score(75.0), Grade::B);
}

#[test]
fn grade_c_for_60_to_69() {
    assert_eq!(Grade::from_score(65.0), Grade::C);
}

#[test]
fn grade_d_for_40_to_59() {
    assert_eq!(Grade::from_score(50.0), Grade::D);
}

#[test]
fn grade_f_for_below_40() {
    assert_eq!(Grade::from_score(20.0), Grade::F);
    assert_eq!(Grade::from_score(0.0), Grade::F);
}

// ── Score calculation ───────────────────────────────────────────────

#[test]
fn all_pass_gives_100() {
    let results = vec![
        make_result(Dimension::Foundation, Severity::Critical, RuleStatus::Pass),
        make_result(Dimension::Foundation, Severity::High, RuleStatus::Pass),
        make_result(Dimension::ContextEfficiency, Severity::High, RuleStatus::Pass),
        make_result(Dimension::Navigation, Severity::Medium, RuleStatus::Pass),
        make_result(Dimension::Tooling, Severity::Low, RuleStatus::Pass),
        make_result(Dimension::CodeQuality, Severity::Medium, RuleStatus::Pass),
    ];
    let pt = make_project_type();
    let sc = calculate(results, &pt);
    assert_eq!(sc.total_score, 100.0);
    assert_eq!(sc.grade, Grade::APlus);
}

#[test]
fn critical_failure_caps_dimension_at_30() {
    let results = vec![
        make_result(Dimension::Foundation, Severity::Critical, RuleStatus::Fail("missing".into())),
        make_result(Dimension::Foundation, Severity::High, RuleStatus::Pass),
        make_result(Dimension::ContextEfficiency, Severity::High, RuleStatus::Pass),
        make_result(Dimension::Navigation, Severity::Medium, RuleStatus::Pass),
        make_result(Dimension::Tooling, Severity::Low, RuleStatus::Pass),
        make_result(Dimension::CodeQuality, Severity::Medium, RuleStatus::Pass),
    ];
    let pt = make_project_type();
    let sc = calculate(results, &pt);

    let foundation = sc.dimensions.iter().find(|d| d.dimension == Dimension::Foundation).unwrap();
    assert!(foundation.capped, "Foundation should be capped");
    assert!(foundation.score <= 30.0, "Foundation score should be ≤30, got {}", foundation.score);
}

#[test]
fn high_failure_deducts_20() {
    let results = vec![
        make_result(Dimension::Foundation, Severity::High, RuleStatus::Fail("missing".into())),
        make_result(Dimension::ContextEfficiency, Severity::High, RuleStatus::Pass),
        make_result(Dimension::Navigation, Severity::Medium, RuleStatus::Pass),
        make_result(Dimension::Tooling, Severity::Low, RuleStatus::Pass),
        make_result(Dimension::CodeQuality, Severity::Medium, RuleStatus::Pass),
    ];
    let pt = make_project_type();
    let sc = calculate(results, &pt);

    let foundation = sc.dimensions.iter().find(|d| d.dimension == Dimension::Foundation).unwrap();
    assert_eq!(foundation.score, 80.0); // 100 - 20
}

#[test]
fn medium_failure_deducts_10() {
    let results = vec![
        make_result(Dimension::Foundation, Severity::Medium, RuleStatus::Fail("x".into())),
    ];
    let pt = make_project_type();
    let sc = calculate(results, &pt);

    let foundation = sc.dimensions.iter().find(|d| d.dimension == Dimension::Foundation).unwrap();
    assert_eq!(foundation.score, 90.0); // 100 - 10
}

#[test]
fn low_failure_deducts_5() {
    let results = vec![
        make_result(Dimension::Foundation, Severity::Low, RuleStatus::Fail("x".into())),
    ];
    let pt = make_project_type();
    let sc = calculate(results, &pt);

    let foundation = sc.dimensions.iter().find(|d| d.dimension == Dimension::Foundation).unwrap();
    assert_eq!(foundation.score, 95.0); // 100 - 5
}

#[test]
fn warning_deducts_half() {
    let results = vec![
        make_result(Dimension::Foundation, Severity::High, RuleStatus::Warn("x".into())),
    ];
    let pt = make_project_type();
    let sc = calculate(results, &pt);

    let foundation = sc.dimensions.iter().find(|d| d.dimension == Dimension::Foundation).unwrap();
    assert_eq!(foundation.score, 90.0); // 100 - (20/2)
}

#[test]
fn skip_does_not_deduct() {
    let results = vec![
        make_result(Dimension::Foundation, Severity::Critical, RuleStatus::Skip),
    ];
    let pt = make_project_type();
    let sc = calculate(results, &pt);

    let foundation = sc.dimensions.iter().find(|d| d.dimension == Dimension::Foundation).unwrap();
    assert_eq!(foundation.score, 100.0);
    assert!(!foundation.capped);
}

#[test]
fn multiple_failures_accumulate() {
    let results = vec![
        make_result(Dimension::Foundation, Severity::High, RuleStatus::Fail("a".into())),
        make_result(Dimension::Foundation, Severity::High, RuleStatus::Fail("b".into())),
        make_result(Dimension::Foundation, Severity::Medium, RuleStatus::Fail("c".into())),
    ];
    let pt = make_project_type();
    let sc = calculate(results, &pt);

    let foundation = sc.dimensions.iter().find(|d| d.dimension == Dimension::Foundation).unwrap();
    assert_eq!(foundation.score, 50.0); // 100 - 20 - 20 - 10
}

#[test]
fn score_floors_at_zero() {
    let results = vec![
        make_result(Dimension::Foundation, Severity::High, RuleStatus::Fail("a".into())),
        make_result(Dimension::Foundation, Severity::High, RuleStatus::Fail("b".into())),
        make_result(Dimension::Foundation, Severity::High, RuleStatus::Fail("c".into())),
        make_result(Dimension::Foundation, Severity::High, RuleStatus::Fail("d".into())),
        make_result(Dimension::Foundation, Severity::High, RuleStatus::Fail("e".into())),
        make_result(Dimension::Foundation, Severity::High, RuleStatus::Fail("f".into())),
    ];
    let pt = make_project_type();
    let sc = calculate(results, &pt);

    let foundation = sc.dimensions.iter().find(|d| d.dimension == Dimension::Foundation).unwrap();
    assert_eq!(foundation.score, 0.0); // 100 - 120 → floored at 0
}

// ── Weight tables ───────────────────────────────────────────────────

#[test]
fn standard_weights_sum_to_100() {
    let pt = make_project_type();
    let weights = claude_native::scoring::weights::get_weights(&pt);
    let sum: f64 = weights.values().sum();
    assert!((sum - 1.0).abs() < 0.001, "Weights sum to {sum}, expected 1.0");
}

#[test]
fn monorepo_weights_sum_to_100() {
    let pt = ProjectType {
        primary: PrimaryType::Monorepo,
        flags: ProjectFlags::default(),
        languages: vec![],
    };
    let weights = claude_native::scoring::weights::get_weights(&pt);
    let sum: f64 = weights.values().sum();
    assert!((sum - 1.0).abs() < 0.001, "Monorepo weights sum to {sum}");
}

#[test]
fn micro_repo_weights_sum_to_100() {
    let pt = ProjectType {
        primary: PrimaryType::MicroRepo,
        flags: ProjectFlags::default(),
        languages: vec![],
    };
    let weights = claude_native::scoring::weights::get_weights(&pt);
    let sum: f64 = weights.values().sum();
    assert!((sum - 1.0).abs() < 0.001, "Micro-repo weights sum to {sum}");
}

#[test]
fn suggestions_are_sorted_by_priority() {
    let results = vec![
        RuleResult {
            rule_id: "1".into(),
            name: "r1".into(),
            dimension: Dimension::Foundation,
            severity: Severity::High,
            status: RuleStatus::Fail("x".into()),
            suggestion: Some(Suggestion {
                priority: SuggestionPriority::NiceToHave,
                title: "nice".into(),
                description: "nice".into(),
                effort: Effort::Hour,
            }),
        },
        RuleResult {
            rule_id: "2".into(),
            name: "r2".into(),
            dimension: Dimension::Foundation,
            severity: Severity::High,
            status: RuleStatus::Fail("y".into()),
            suggestion: Some(Suggestion {
                priority: SuggestionPriority::QuickWin,
                title: "quick".into(),
                description: "quick".into(),
                effort: Effort::Minutes,
            }),
        },
    ];
    let pt = make_project_type();
    let sc = calculate(results, &pt);
    assert_eq!(sc.suggestions[0].priority, SuggestionPriority::QuickWin);
    assert_eq!(sc.suggestions[1].priority, SuggestionPriority::NiceToHave);
}
