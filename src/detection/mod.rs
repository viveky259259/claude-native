pub mod domain;
pub mod signals;
pub mod signals_backend;

use crate::scan::ProjectContext;

/// Primary project type detected from heuristics
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimaryType {
    Standard,
    Monorepo,
    MicroRepo,
    Mobile(MobileFramework),
    Frontend(FrontendFramework),
    Backend(BackendFramework),
    IaC(IaCTool),
    Serverless(ServerlessPlatform),
    ML,
    CodegenHeavy,
    DocSite(DocFramework),
    GameDev(GameEngine),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MobileFramework {
    Flutter,
    ReactNative,
    IosNative,
    AndroidNative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrontendFramework {
    NextJs,
    Nuxt,
    Angular,
    VueVite,
    CreateReactApp,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendFramework {
    Django,
    Rails,
    Express,
    GoService,
    RustService,
    Phoenix,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IaCTool {
    Terraform,
    Helm,
    Kustomize,
    Pulumi,
    AwsCdk,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerlessPlatform {
    ServerlessFramework,
    AwsSam,
    CloudflareWorkers,
    VercelFunctions,
    NetlifyFunctions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocFramework {
    Docusaurus,
    MkDocs,
    Hugo,
    VuePress,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEngine {
    Unity,
    Godot,
    Bevy,
    Unreal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Dart,
    Swift,
    Kotlin,
    Java,
    Ruby,
    CSharp,
    Elixir,
    Cpp,
    Other(String),
}

/// Compound flags that overlay on PrimaryType
#[derive(Debug, Clone, Default)]
pub struct ProjectFlags {
    pub is_polyglot: bool,
    pub is_legacy: bool,
}

/// Full detection result
#[derive(Debug, Clone)]
pub struct ProjectType {
    pub primary: PrimaryType,
    pub flags: ProjectFlags,
    pub languages: Vec<Language>,
}

impl std::fmt::Display for ProjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let primary = match &self.primary {
            PrimaryType::Standard => "Standard".to_string(),
            PrimaryType::Monorepo => "Monorepo".to_string(),
            PrimaryType::MicroRepo => "Micro-repo".to_string(),
            PrimaryType::Mobile(fw) => format!("Mobile ({fw:?})"),
            PrimaryType::Frontend(fw) => format!("Frontend ({fw:?})"),
            PrimaryType::Backend(fw) => format!("Backend ({fw:?})"),
            PrimaryType::IaC(tool) => format!("IaC ({tool:?})"),
            PrimaryType::Serverless(p) => format!("Serverless ({p:?})"),
            PrimaryType::ML => "Data Science / ML".to_string(),
            PrimaryType::CodegenHeavy => "Code Generation Heavy".to_string(),
            PrimaryType::DocSite(fw) => format!("Documentation Site ({fw:?})"),
            PrimaryType::GameDev(eng) => format!("Game Development ({eng:?})"),
        };

        let mut flags = Vec::new();
        if self.flags.is_polyglot {
            flags.push("Polyglot");
        }
        if self.flags.is_legacy {
            flags.push("Legacy");
        }

        if flags.is_empty() {
            write!(f, "{primary}")
        } else {
            write!(f, "{primary} + {}", flags.join(" + "))
        }
    }
}

/// Detect the project type from a fully scanned ProjectContext.
pub fn detect(ctx: &ProjectContext) -> ProjectType {
    let languages = signals::detect_languages(ctx);
    let flags = detect_flags(ctx, &languages);

    // Phase 1: Structure check — monorepo
    if signals::is_monorepo(ctx) {
        return ProjectType {
            primary: PrimaryType::Monorepo,
            flags,
            languages,
        };
    }

    // Phase 2: Domain-specific markers (before micro-repo, since a small
    // Flutter/Django/Terraform project is still domain-specific, not a generic micro-repo)
    if let Some(primary) = domain::detect_domain(ctx) {
        return ProjectType {
            primary,
            flags,
            languages,
        };
    }

    // Phase 3: Micro-repo (only if no domain marker matched)
    if signals::is_micro_repo(ctx) {
        return ProjectType {
            primary: PrimaryType::MicroRepo,
            flags,
            languages,
        };
    }

    ProjectType {
        primary: PrimaryType::Standard,
        flags,
        languages,
    }
}

fn detect_flags(ctx: &ProjectContext, languages: &[Language]) -> ProjectFlags {
    let is_polyglot = languages.len() >= 2;

    let is_legacy = ctx.test_files.is_empty()
        && ctx.source_file_count() > 10
        && ctx.average_source_file_lines() > 300.0
        && !ctx.has_claude_md();

    ProjectFlags {
        is_polyglot,
        is_legacy,
    }
}
