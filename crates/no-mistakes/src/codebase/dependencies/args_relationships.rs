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
        self.workflow_str()
            .unwrap_or_else(|| self.non_workflow_str())
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

    fn non_workflow_str(&self) -> &'static str {
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
            Self::Dotnet => "dotnet",
            Self::Swift => "swift",
            Self::Terraform => "terraform",
            Self::Resource => "resource",
            Self::All => "all",
            Self::Workflow
            | Self::WorkflowJob
            | Self::WorkflowStep
            | Self::WorkflowNeeds
            | Self::WorkflowUses
            | Self::WorkflowRun
            | Self::WorkflowArtifact => unreachable!("workflow relationships are handled first"),
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
