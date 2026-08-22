pub enum Direction {
    Deps,
    Dependents,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, serde::Deserialize, serde::Serialize,
)]
#[clap(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum RelationshipArg {
    Import,
    ImportStatic,
    ImportDynamic,
    ImportType,
    ImportRequire,
    RouteImport,
    Workspace,
    Package,
    Test,
    Route,
    Queue,
    Md,
    Ci,
    Workflow,
    WorkflowJob,
    WorkflowStep,
    WorkflowNeeds,
    WorkflowUses,
    WorkflowRun,
    WorkflowArtifact,
    Http,
    Process,
    Asset,
    React,
    Dotnet,
    Swift,
    Terraform,
    Python,
    Go,
    Rust,
    Ruby,
    Php,
    Java,
    Kotlin,
    Elixir,
    Resource,
    Trpc,
    All,
}

impl RelationshipArg {
    pub fn as_str(&self) -> &'static str {
        self.workflow_str()
            .or_else(|| self.language_str())
            .unwrap_or_else(|| self.core_str())
    }

    fn workflow_str(&self) -> Option<&'static str> {
        match self {
            Self::Workflow => Some("workflow"),
            Self::WorkflowJob => Some("workflow-job"),
            Self::WorkflowStep => Some("workflow-step"),
            Self::WorkflowNeeds => Some("workflow-needs"),
            Self::WorkflowUses => Some("workflow-uses"),
            Self::WorkflowRun => Some("workflow-run"),
            Self::WorkflowArtifact => Some("workflow-artifact"),
            _ => None,
        }
    }

    fn language_str(&self) -> Option<&'static str> {
        match self {
            Self::Dotnet => Some("dotnet"),
            Self::Swift => Some("swift"),
            Self::Terraform => Some("terraform"),
            Self::Python => Some("python"),
            Self::Go => Some("go"),
            Self::Rust => Some("rust"),
            Self::Ruby => Some("ruby"),
            Self::Php => Some("php"),
            Self::Java => Some("java"),
            Self::Kotlin => Some("kotlin"),
            Self::Elixir => Some("elixir"),
            Self::Trpc => Some("trpc"),
            _ => None,
        }
    }

    fn core_str(&self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::ImportStatic => "import-static",
            Self::ImportDynamic => "import-dynamic",
            Self::ImportType => "import-type",
            Self::ImportRequire => "import-require",
            Self::RouteImport => "route-import",
            Self::Workspace => "workspace",
            Self::Package => "package",
            Self::Test => "test",
            Self::Route => "route",
            Self::Queue => "queue",
            Self::Md => "md",
            Self::Ci => "ci",
            Self::Http => "http",
            Self::Process => "process",
            Self::Asset => "asset",
            Self::React => "react",
            Self::Resource => "resource",
            Self::All => "all",
            _ => unreachable!("workflow and language relationships are handled first"),
        }
    }
}

include!("args_relationships_filter.rs");

fn relationships_are_import_only(relationships: &[RelationshipArg]) -> bool {
    !relationships.is_empty()
        && relationships.iter().all(|relationship| {
            matches!(
                relationship,
                RelationshipArg::Import
                    | RelationshipArg::ImportStatic
                    | RelationshipArg::ImportDynamic
                    | RelationshipArg::ImportType
                    | RelationshipArg::ImportRequire
            )
        })
}

/// A resolved entrypoint: a file/module node, plus an optional exported symbol / queue job name.
struct Entrypoint {
    file: PathBuf,
    node: NodeId,
    symbol: Option<String>,
}

pub fn parse_entrypoint(s: &str) -> (PathBuf, Option<String>) {
    match s.split_once('#') {
        Some((file, symbol)) => (PathBuf::from(file), Some(symbol.to_string())),
        None => (PathBuf::from(s), None),
    }
}

pub(crate) fn workflow_node_from_suffix_in(
    interner: &PathInterner,
    file: &Path,
    suffix: &str,
) -> Option<NodeId> {
    parsed_workflow_suffix(suffix).map(|(job, step)| match step {
        Some(step) => NodeId::workflow_step_in(interner, file, job, step),
        None => NodeId::workflow_job_in(interner, file, job),
    })
}

fn parsed_workflow_suffix(suffix: &str) -> Option<(&str, Option<usize>)> {
    let suffix = suffix.strip_prefix("job:")?;
    if let Some((job, step)) = suffix.split_once("/step:") {
        if job.is_empty() {
            return None;
        }
        return Some((job, Some(step.parse().ok()?)));
    }
    (!suffix.is_empty()).then_some((suffix, None))
}

pub(crate) fn trpc_procedure_from_suffix(file: &Path, suffix: &str) -> Option<NodeId> {
    let procedure = suffix.strip_prefix("procedure:")?;
    (!procedure.is_empty()).then(|| NodeId::trpc_procedure(file, procedure))
}
