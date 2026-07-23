pub use crate::codebase::ts_source::SKIP_DIRS;

/// A node in the dependency graph: a source file, external module, or virtual node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeId {
    /// A source file on disk.
    File(PathBuf),
    /// An exported symbol in a source file.
    Symbol { file: PathBuf, symbol: String },
    /// A bare external module specifier that is not resolved to a local file.
    Module(String),
    /// A virtual job node representing one (queue, jobName) pair.
    QueueJob { queue_file: PathBuf, job: String },
    /// A virtual GitHub Actions workflow job.
    WorkflowJob { workflow_file: PathBuf, job: String },
    /// A virtual GitHub Actions workflow step. `step` is zero-based.
    WorkflowStep {
        workflow_file: PathBuf,
        job: String,
        step: usize,
    },
}

impl NodeId {
    /// Return the underlying file path, if this is a `File` node.
    pub fn as_file(&self) -> Option<&Path> {
        match self {
            NodeId::File(p) => Some(p.as_path()),
            NodeId::Symbol { file, .. } => Some(file.as_path()),
            NodeId::Module(_) => None,
            NodeId::QueueJob { .. } => None,
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

    /// Render this node relative to `root` for display.
    pub fn display_name(&self, root: &Path) -> String {
        match self {
            NodeId::File(p) => {
                let rel = p.strip_prefix(root).unwrap_or(p);
                rel.display().to_string()
            }
            NodeId::Symbol { file, symbol } => {
                let rel = file.strip_prefix(root).unwrap_or(file);
                format!("{}#{}", rel.display(), symbol)
            }
            NodeId::Module(specifier) => specifier.clone(),
            NodeId::QueueJob { queue_file, job } => {
                let rel = queue_file
                    .strip_prefix(root)
                    .unwrap_or(queue_file.as_path());
                format!("{}#{}", rel.display(), job)
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

/// A single node in the traversal result.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeEntry {
    /// The graph node (file or virtual queue-job).
    pub node: NodeId,
    /// Traversal depth (1 = direct dep/dependent, 2 = transitive, etc.).
    pub depth: usize,
    /// Edge kinds that led to this node (deduped, sorted).
    pub via: Vec<EdgeKind>,
}

type EdgeMap = HashMap<NodeId, Vec<(NodeId, EdgeKind)>>;

// An edge in both directions: (from, to, kind).
type Edge = (NodeId, NodeId, EdgeKind);

/// Provenance for one statically resolved runtime-resource call.
///
/// The graph deliberately keeps this outside the generic adjacency index: the
/// relationship remains a normal typed edge while callers that need debug or
/// test-impact explanations can retrieve every call site that produced it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceCallSite {
    pub call_kind: String,
    /// One-based source line containing the call expression.
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

/// A dynamic runtime-resource call that deliberately did not create an edge.
/// Kept alongside the canonical graph so impact rendering can surface only
/// diagnostics that are relevant to a selected path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceGraphDiagnostic {
    pub consumer: PathBuf,
    pub kind: crate::codebase::ts_resources::ResourceDiagnosticKind,
    /// One-based source line containing the dynamic call expression.
    pub line: usize,
}

type ParsedImports<'a> = Vec<(
    &'a PathBuf,
    &'a crate::codebase::ts_source::facts::TsFileFacts,
    HashSet<String>,
)>;
