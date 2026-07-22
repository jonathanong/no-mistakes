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
