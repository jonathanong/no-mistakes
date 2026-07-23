pub use crate::codebase::ts_source::SKIP_DIRS;

/// A node in the dependency graph: a source file, external module, or virtual node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeId {
    File(PathBuf),
    Symbol { file: PathBuf, symbol: String },
    Module(String),
    QueueJob { queue_file: PathBuf, job: String },
    WorkflowJob { workflow_file: PathBuf, job: String },
    WorkflowStep {
        workflow_file: PathBuf,
        job: String,
        step: usize,
    },
}

impl NodeId {
    pub fn as_file(&self) -> Option<&Path> {
        match self {
            NodeId::File(p) => Some(p.as_path()),
            NodeId::Symbol { file, .. } => Some(file.as_path()),
            NodeId::Module(_) | NodeId::QueueJob { .. } => None,
            NodeId::WorkflowJob { .. } | NodeId::WorkflowStep { .. } => None,
        }
    }

    fn is_in_file_universe(&self, universe: &HashSet<PathBuf>) -> bool {
        match self {
            Self::File(path) | Self::Symbol { file: path, .. } => universe.contains(path),
            Self::Module(_) => true,
            Self::QueueJob { queue_file, .. } => universe.contains(queue_file),
            Self::WorkflowJob { workflow_file, .. } | Self::WorkflowStep { workflow_file, .. } => {
                universe.contains(workflow_file)
            }
        }
    }

    pub fn display_name(&self, root: &Path) -> String {
        match self {
            NodeId::File(p) => {
                let rel = p.strip_prefix(root).unwrap_or(p);
                rel.display().to_string()
            }
            NodeId::Symbol { file, symbol } => {
                let rel = file.strip_prefix(root).unwrap_or(file);
                format!("{}#{symbol}", rel.display())
            }
            NodeId::Module(specifier) => specifier.clone(),
            NodeId::QueueJob { queue_file, job } => {
                let rel = queue_file
                    .strip_prefix(root)
                    .unwrap_or(queue_file.as_path());
                format!("{}#{job}", rel.display())
            }
            NodeId::WorkflowJob { workflow_file, job } => {
                let rel = workflow_file
                    .strip_prefix(root)
                    .unwrap_or(workflow_file.as_path());
                format!("{}#job:{job}", rel.display())
            }
            NodeId::WorkflowStep {
                workflow_file,
                job,
                step,
            } => {
                let rel = workflow_file
                    .strip_prefix(root)
                    .unwrap_or(workflow_file.as_path());
                format!("{}#job:{job}/step:{step}", rel.display())
            }
        }
    }
}

include!("types_edges.rs");

#[derive(Debug, Clone, PartialEq)]
pub struct NodeEntry {
    pub node: NodeId,
    pub depth: usize,
    pub via: Vec<EdgeKind>,
}

type EdgeMap = HashMap<NodeId, Vec<(NodeId, EdgeKind)>>;
type Edge = (NodeId, NodeId, EdgeKind);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceCallSite {
    pub call_kind: String,
    pub line: usize,
}

impl PartialOrd for ResourceCallSite {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ResourceCallSite {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.line, &self.call_kind).cmp(&(other.line, &other.call_kind))
    }
}

type ResourceEdgeDetails = HashMap<(PathBuf, PathBuf), Vec<ResourceCallSite>>;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceGraphDiagnostic {
    pub consumer: PathBuf,
    pub kind: crate::codebase::ts_resources::ResourceDiagnosticKind,
    pub line: usize,
}

type ParsedImports<'a> = Vec<(
    &'a PathBuf,
    &'a crate::codebase::ts_source::facts::TsFileFacts,
    HashSet<String>,
)>;
