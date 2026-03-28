pub mod weights;

use std::collections::HashMap;

use crate::detection::ProjectType;
use crate::rules::{Dimension, Effort, RuleResult, RuleStatus, Severity, Suggestion, SuggestionPriority};

/// Grade from A+ to F
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Grade {
    F,
    D,
    C,
    B,
    A,
    APlus,
}

impl Grade {
    pub fn from_score(score: f64) -> Self {
        match score as u32 {
            90..=100 => Grade::APlus,
            80..=89 => Grade::A,
            70..=79 => Grade::B,
            60..=69 => Grade::C,
            40..=59 => Grade::D,
            _ => Grade::F,
        }
    }
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Grade::APlus => write!(f, "A+"),
            Grade::A => write!(f, "A"),
            Grade::B => write!(f, "B"),
            Grade::C => write!(f, "C"),
            Grade::D => write!(f, "D"),
            Grade::F => write!(f, "F"),
        }
    }
}

impl Grade {
    pub fn description(&self) -> &str {
        match self {
            Grade::APlus => "Fully Claude Native — optimized for AI-assisted development",
            Grade::A => "Claude Native — well set up with minor improvements possible",
            Grade::B => "Claude Friendly — good foundation, notable gaps",
            Grade::C => "Claude Compatible — works but significant optimization possible",
            Grade::D => "Claude Hostile — major friction, high token waste",
            Grade::F => "Not Claude Native — needs fundamental restructuring",
        }
    }
}

/// Score for a single dimension
#[derive(Debug, Clone)]
pub struct DimensionScore {
    pub dimension: Dimension,
    pub score: f64,
    pub weight: f64,
    pub rules_passed: usize,
    pub rules_failed: usize,
    pub rules_warned: usize,
    pub rules_skipped: usize,
    pub capped: bool,
}

/// The complete scorecard
#[derive(Debug, Clone)]
pub struct Scorecard {
    pub project_type: ProjectType,
    pub dimensions: Vec<DimensionScore>,
    pub total_score: f64,
    pub grade: Grade,
    pub rule_results: Vec<RuleResult>,
    pub suggestions: Vec<Suggestion>,
}

const ALL_DIMENSIONS: [Dimension; 5] = [
    Dimension::Foundation,
    Dimension::ContextEfficiency,
    Dimension::Navigation,
    Dimension::Tooling,
    Dimension::CodeQuality,
];

/// Calculate the scorecard from rule results.
pub fn calculate(results: Vec<RuleResult>, project_type: &ProjectType) -> Scorecard {
    let weight_table = weights::get_weights(project_type);
    let mut by_dimension: HashMap<Dimension, Vec<&RuleResult>> = HashMap::new();
    for r in &results {
        by_dimension.entry(r.dimension).or_default().push(r);
    }

    let dim_scores: Vec<DimensionScore> = ALL_DIMENSIONS.iter()
        .map(|dim| score_dimension(*dim, &by_dimension, &weight_table))
        .collect();

    let total_score = dim_scores.iter().map(|d| d.score * d.weight).sum::<f64>().clamp(0.0, 100.0);
    let grade = Grade::from_score(total_score);
    let suggestions = collect_suggestions(&results, total_score);

    Scorecard { project_type: project_type.clone(), dimensions: dim_scores, total_score, grade, rule_results: results, suggestions }
}

fn score_dimension(
    dim: Dimension,
    by_dimension: &HashMap<Dimension, Vec<&RuleResult>>,
    weight_table: &HashMap<Dimension, f64>,
) -> DimensionScore {
    let dim_results = by_dimension.get(&dim).cloned().unwrap_or_default();
    let weight = weight_table.get(&dim).copied().unwrap_or(0.2);
    let mut score = 100.0_f64;
    let mut capped = false;
    let (mut passed, mut failed, mut warned, mut skipped) = (0, 0, 0, 0);

    for r in &dim_results {
        match &r.status {
            RuleStatus::Pass => passed += 1,
            RuleStatus::Skip => skipped += 1,
            RuleStatus::Warn(_) => { warned += 1; score -= r.severity.deduction() / 2.0; }
            RuleStatus::Fail(_) => {
                failed += 1;
                if r.severity == Severity::Critical { capped = true; }
                else { score -= r.severity.deduction(); }
            }
        }
    }

    if capped { score = score.min(30.0); }
    score = score.clamp(0.0, 100.0);

    DimensionScore { dimension: dim, score, weight, rules_passed: passed, rules_failed: failed, rules_warned: warned, rules_skipped: skipped, capped }
}

fn collect_suggestions(results: &[RuleResult], total_score: f64) -> Vec<Suggestion> {
    let mut suggestions: Vec<Suggestion> = results.iter()
        .filter_map(|r| r.suggestion.clone())
        .collect();
    suggestions.sort_by_key(|s| s.priority);

    // When score is below C grade, prepend --init as the #1 suggestion
    if total_score < 60.0 {
        suggestions.insert(0, Suggestion {
            priority: SuggestionPriority::QuickWin,
            title: "Run `claude-native --init` to bootstrap your project".into(),
            description: "This single command generates CLAUDE.md, .claudeignore, and .claude/settings.json tailored to your detected project type. Typically jumps score by 20-30 points.".into(),
            effort: Effort::Minutes,
        });
    }

    suggestions
}
