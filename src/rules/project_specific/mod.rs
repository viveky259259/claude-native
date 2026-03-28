pub mod monorepo;
pub mod micro_repo;
pub mod mobile;
pub mod frontend;
pub mod backend;
pub mod iac;
pub mod serverless;
pub mod ml;
pub mod codegen;
pub mod polyglot;
pub mod legacy;
pub mod doc_site;
pub mod game_dev;

use crate::rules::Rule;

/// Collect all project-specific rules
pub fn project_specific_rules() -> Vec<Box<dyn Rule>> {
    vec![
        // Monorepo (M1-M7)
        Box::new(monorepo::RootClaudeMdThin),
        Box::new(monorepo::PerPackageClaudeMd),
        Box::new(monorepo::WorkspaceOutputsIgnored),
        Box::new(monorepo::PathScopedRulesPerPackage),
        Box::new(monorepo::PerPackageTestCommands),
        Box::new(monorepo::WorkspaceDepsNavigable),
        Box::new(monorepo::SharedToolingConfig),
        // Micro-repo (μ1-μ4)
        Box::new(micro_repo::ReadmeIsPrimary),
        Box::new(micro_repo::ComprehensiveTests),
        Box::new(micro_repo::ManifestComplete),
        Box::new(micro_repo::ExamplesExist),
        // Mobile (MOB1-MOB5)
        Box::new(mobile::PlatformBuildDirsIgnored),
        Box::new(mobile::GeneratedCodeIgnored),
        Box::new(mobile::PlatformCommands),
        Box::new(mobile::BinaryAssetsExcluded),
        Box::new(mobile::LightweightVerification),
        // Frontend (FE1-FE4)
        Box::new(frontend::BuildCacheIgnored),
        Box::new(frontend::SourceMapsIgnored),
        Box::new(frontend::EnvExampleExists),
        Box::new(frontend::TypeCheckBeforeBuild),
        // Backend (BE1-BE6)
        Box::new(backend::MigrationHistoryManageable),
        Box::new(backend::DatabaseFilesIgnored),
        Box::new(backend::VirtualEnvsIgnored),
        Box::new(backend::DataAccessDocumented),
        Box::new(backend::ApiSpecExists),
        Box::new(backend::LogFilesIgnored),
        // IaC (IAC1-IAC5)
        Box::new(iac::StateFilesBlocked),
        Box::new(iac::PlanOutputFiltered),
        Box::new(iac::ProviderCacheIgnored),
        Box::new(iac::SecretsExternal),
        Box::new(iac::ModuleConventions),
        // Serverless (SLS1-SLS4)
        Box::new(serverless::DeployArtifactsIgnored),
        Box::new(serverless::FunctionsFocused),
        Box::new(serverless::TestEventsExist),
        Box::new(serverless::NoCloudCreds),
        // ML (DS1-DS5)
        Box::new(ml::NotebooksNotPrimary),
        Box::new(ml::ModelFilesIgnored),
        Box::new(ml::DatasetFilesIgnored),
        Box::new(ml::ExperimentWorkflowDocumented),
        Box::new(ml::RequirementsPinned),
        // Codegen (GEN1-GEN4)
        Box::new(codegen::GeneratedDirsIgnored),
        Box::new(codegen::EditSpecsNotGenerated),
        Box::new(codegen::RegenCommandDocumented),
        Box::new(codegen::AutoRegenHook),
        // Polyglot (POLY1-POLY4)
        Box::new(polyglot::PerLanguageClaudeMd),
        Box::new(polyglot::RootClaudeMdAgnostic),
        Box::new(polyglot::IndependentBuildTest),
        Box::new(polyglot::AllRuntimesIgnored),
        // Legacy (LEG1-LEG5)
        Box::new(legacy::DocumentsTheMess),
        Box::new(legacy::CorrectPatternsIdentified),
        Box::new(legacy::TestsForModifiedCode),
        Box::new(legacy::DeadCodeFlagged),
        Box::new(legacy::MegaFilesDocumented),
        // Doc Site (DOC1-DOC3)
        Box::new(doc_site::BuildOutputIgnored),
        Box::new(doc_site::MarkdownSourceFocus),
        Box::new(doc_site::NavigationDocumented),
        // Game Dev (GAME1-GAME4)
        Box::new(game_dev::BinaryAssetsIgnored),
        Box::new(game_dev::EditorMetadataIgnored),
        Box::new(game_dev::ScriptsArePrimary),
        Box::new(game_dev::GameLogicTests),
    ]
}
