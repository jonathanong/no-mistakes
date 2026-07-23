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
    Resource,
    All,
}

impl RelationshipArg {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationshipArg::Import => "import",
            RelationshipArg::ImportStatic => "import-static",
            RelationshipArg::ImportDynamic => "import-dynamic",
            RelationshipArg::ImportType => "import-type",
            RelationshipArg::ImportRequire => "import-require",
            RelationshipArg::RouteImport => "route-import",
            RelationshipArg::Workspace => "workspace",
            RelationshipArg::Package => "package",
            RelationshipArg::Test => "test",
            RelationshipArg::Route => "route",
            RelationshipArg::Queue => "queue",
            RelationshipArg::Md => "md",
            RelationshipArg::Ci => "ci",
            RelationshipArg::Workflow => "workflow",
            RelationshipArg::WorkflowJob => "workflow-job",
            RelationshipArg::WorkflowStep => "workflow-step",
            RelationshipArg::WorkflowNeeds => "workflow-needs",
            RelationshipArg::WorkflowUses => "workflow-uses",
            RelationshipArg::WorkflowRun => "workflow-run",
            RelationshipArg::WorkflowArtifact => "workflow-artifact",
            RelationshipArg::Http => "http",
            RelationshipArg::Process => "process",
            RelationshipArg::Asset => "asset",
            RelationshipArg::React => "react",
            RelationshipArg::Dotnet => "dotnet",
            RelationshipArg::Swift => "swift",
            RelationshipArg::Terraform => "terraform",
            RelationshipArg::Resource => "resource",
            RelationshipArg::All => "all",
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

pub(crate) fn workflow_node_from_suffix(file: &Path, suffix: &str) -> Option<NodeId> {
    let suffix = suffix.strip_prefix("job:")?;
    if let Some((job, step)) = suffix.split_once("/step:") {
        if job.is_empty() {
            return None;
        }
        return Some(NodeId::WorkflowStep {
            workflow_file: file.to_path_buf(),
            job: job.to_string(),
            step: step.parse().ok()?,
        });
    }
    (!suffix.is_empty()).then(|| NodeId::WorkflowJob {
        workflow_file: file.to_path_buf(),
        job: suffix.to_string(),
    })
}
