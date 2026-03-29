pub mod foundation;
pub mod foundation_extra;
pub mod context;
pub mod drift;
pub mod context_extra;
pub mod navigation;
pub mod navigation_extra;
pub mod tooling;
pub mod quality;
pub mod quality_extra;
pub mod project_specific;

use crate::detection::ProjectType;
use crate::scan::ProjectContext;

/// Scoring dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dimension {
    Foundation,
    ContextEfficiency,
    Navigation,
    Tooling,
    CodeQuality,
}

impl std::fmt::Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dimension::Foundation => write!(f, "Foundation"),
            Dimension::ContextEfficiency => write!(f, "Context Efficiency"),
            Dimension::Navigation => write!(f, "Navigation"),
            Dimension::Tooling => write!(f, "Tooling"),
            Dimension::CodeQuality => write!(f, "Code Quality"),
        }
    }
}

/// Rule severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn deduction(&self) -> f64 {
        match self {
            Severity::Low => 5.0,
            Severity::Medium => 10.0,
            Severity::High => 20.0,
            Severity::Critical => 0.0, // caps at 30 instead
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "LOW"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::High => write!(f, "HIGH"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Outcome of a single rule check
#[derive(Debug, Clone)]
pub enum RuleStatus {
    Pass,
    Warn(String),
    Fail(String),
    Skip,
}

impl RuleStatus {
    pub fn is_failure(&self) -> bool {
        matches!(self, RuleStatus::Fail(_))
    }

    pub fn is_warning(&self) -> bool {
        matches!(self, RuleStatus::Warn(_))
    }

    pub fn is_pass(&self) -> bool {
        matches!(self, RuleStatus::Pass)
    }

    pub fn is_skip(&self) -> bool {
        matches!(self, RuleStatus::Skip)
    }
}

/// Suggestion priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SuggestionPriority {
    QuickWin,
    HighImpact,
    NiceToHave,
}

impl std::fmt::Display for SuggestionPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuggestionPriority::QuickWin => write!(f, "Quick Win"),
            SuggestionPriority::HighImpact => write!(f, "High Impact"),
            SuggestionPriority::NiceToHave => write!(f, "Nice to Have"),
        }
    }
}

/// Effort estimate
#[derive(Debug, Clone, Copy)]
pub enum Effort {
    Minutes,
    Hour,
    HalfDay,
}

impl std::fmt::Display for Effort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effort::Minutes => write!(f, "~2 min"),
            Effort::Hour => write!(f, "~1 hour"),
            Effort::HalfDay => write!(f, "~half day"),
        }
    }
}

/// A suggestion attached to a failed/warned rule
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub priority: SuggestionPriority,
    pub title: String,
    pub description: String,
    pub effort: Effort,
}

/// Complete result of evaluating one rule
#[derive(Debug, Clone)]
pub struct RuleResult {
    pub rule_id: String,
    pub name: String,
    pub dimension: Dimension,
    pub severity: Severity,
    pub status: RuleStatus,
    pub suggestion: Option<Suggestion>,
}

/// The rule trait — each check implements this
pub trait Rule: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn dimension(&self) -> Dimension;
    fn severity(&self) -> Severity;

    /// Whether this rule applies to the detected project type.
    fn applies_to(&self, _project_type: &ProjectType) -> bool {
        true
    }

    /// Execute the check.
    fn check(&self, ctx: &ProjectContext) -> RuleResult;

    /// Helper to produce a pass result.
    fn pass(&self) -> RuleResult {
        RuleResult {
            rule_id: self.id().to_string(),
            name: self.name().to_string(),
            dimension: self.dimension(),
            severity: self.severity(),
            status: RuleStatus::Pass,
            suggestion: None,
        }
    }

    /// Helper to produce a fail result with suggestion.
    fn fail(&self, reason: &str, suggestion: Suggestion) -> RuleResult {
        RuleResult {
            rule_id: self.id().to_string(),
            name: self.name().to_string(),
            dimension: self.dimension(),
            severity: self.severity(),
            status: RuleStatus::Fail(reason.to_string()),
            suggestion: Some(suggestion),
        }
    }

    /// Helper to produce a warn result with suggestion.
    fn warn(&self, reason: &str, suggestion: Suggestion) -> RuleResult {
        RuleResult {
            rule_id: self.id().to_string(),
            name: self.name().to_string(),
            dimension: self.dimension(),
            severity: self.severity(),
            status: RuleStatus::Warn(reason.to_string()),
            suggestion: Some(suggestion),
        }
    }

    /// Helper to produce a skip result.
    fn skip(&self) -> RuleResult {
        RuleResult {
            rule_id: self.id().to_string(),
            name: self.name().to_string(),
            dimension: self.dimension(),
            severity: self.severity(),
            status: RuleStatus::Skip,
            suggestion: None,
        }
    }
}

/// Collect all rules (base + project-specific)
pub fn all_rules() -> Vec<Box<dyn Rule>> {
    let mut rules: Vec<Box<dyn Rule>> = Vec::new();

    // Foundation (1.1 - 1.7)
    rules.push(Box::new(foundation::ClaudeMdExists));
    rules.push(Box::new(foundation::ClaudeMdConcise));
    rules.push(Box::new(foundation::ClaudeMdActionable));
    rules.push(Box::new(foundation::ClaudeMdHasCommands));
    rules.push(Box::new(foundation::ClaudeignoreExists));
    rules.push(Box::new(foundation::ClaudeDirExists));
    rules.push(Box::new(foundation::SettingsJsonExists));
    rules.push(Box::new(foundation_extra::AgentsMdExists));

    // Context drift (6.1 - 6.2)
    rules.push(Box::new(drift::BuildCommandMatchesManifest));
    rules.push(Box::new(drift::ReferencedFilesExist));

    // Context Efficiency (2.1 - 2.7)
    rules.push(Box::new(context::NoMegaFiles));
    rules.push(Box::new(context::NoMegaFunctions));
    rules.push(Box::new(context::LockFilesIgnored));
    rules.push(Box::new(context::GeneratedFilesIgnored));
    rules.push(Box::new(context_extra::NoSecretsInRepo));
    rules.push(Box::new(context_extra::ReadmeExistsAndConcise));
    rules.push(Box::new(context_extra::SubdirClaudeMd));

    // Navigation (3.1 - 3.7)
    rules.push(Box::new(navigation::ClearDirectoryStructure));
    rules.push(Box::new(navigation::ConsistentNaming));
    rules.push(Box::new(navigation::ObviousEntryPoints));
    rules.push(Box::new(navigation_extra::ClearModuleBoundaries));
    rules.push(Box::new(navigation_extra::PredictableTestLocations));
    rules.push(Box::new(navigation_extra::NoDeepNesting));
    rules.push(Box::new(navigation_extra::DescriptiveNames));

    // Tooling (4.1 - 4.6)
    rules.push(Box::new(tooling::McpServersConfigured));
    rules.push(Box::new(tooling::AutoFormatHook));
    rules.push(Box::new(tooling::DangerousOpProtection));
    rules.push(Box::new(tooling::CustomSkills));
    rules.push(Box::new(tooling::PermissionAllowList));
    rules.push(Box::new(tooling::PathScopedRules));
    rules.push(Box::new(tooling::SubagentConfig));

    // Code Quality (5.1 - 5.8)
    rules.push(Box::new(quality::TypeAnnotationsExist));
    rules.push(Box::new(quality::TestsExist));
    rules.push(Box::new(quality_extra::DescriptiveTestNames));
    rules.push(Box::new(quality_extra::ConsistentPatterns));
    rules.push(Box::new(quality_extra::CommentsExplainWhy));
    rules.push(Box::new(quality_extra::NoDeadCode));
    rules.push(Box::new(quality_extra::DependenciesDocumented));
    rules.push(Box::new(quality_extra::CiCdExists));

    // Project-specific rules (filtered by applies_to at runtime)
    rules.extend(project_specific::project_specific_rules());

    rules
}
