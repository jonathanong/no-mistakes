pub(crate) fn intern_node_path(path: impl AsRef<Path>) -> Arc<Path> {
    Arc::from(path.as_ref())
}

pub(crate) fn intern_node_str(value: impl AsRef<str>) -> Arc<str> {
    Arc::from(value.as_ref())
}

impl NodeId {
    /// Construct a file node. Use in expressions only — match `NodeId::File(path)`.
    pub fn file(path: impl AsRef<Path>) -> Self {
        Self::File(intern_node_path(path))
    }

    /// Construct a symbol node. Use in expressions only — match `NodeId::Symbol { .. }`.
    pub fn symbol(path: impl AsRef<Path>, symbol: impl AsRef<str>) -> Self {
        Self::Symbol {
            file: intern_node_path(path),
            symbol: intern_node_str(symbol),
        }
    }

    /// Construct a queue-job node. Use in expressions only — match `NodeId::QueueJob { .. }`.
    pub fn queue_job(path: impl AsRef<Path>, job: impl AsRef<str>) -> Self {
        Self::QueueJob {
            queue_file: intern_node_path(path),
            job: intern_node_str(job),
        }
    }

    /// Construct a workflow-job node. Use in expressions only — match `NodeId::WorkflowJob { .. }`.
    pub fn workflow_job(path: impl AsRef<Path>, job: impl AsRef<str>) -> Self {
        Self::WorkflowJob {
            workflow_file: intern_node_path(path),
            job: intern_node_str(job),
        }
    }

    /// Construct a workflow-step node. Use in expressions only — match `NodeId::WorkflowStep { .. }`.
    pub fn workflow_step(path: impl AsRef<Path>, job: impl AsRef<str>, step: usize) -> Self {
        Self::WorkflowStep {
            workflow_file: intern_node_path(path),
            job: intern_node_str(job),
            step,
        }
    }

    /// Path for any path-bearing variant, including queue/workflow nodes.
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            NodeId::File(path) => Some(path.as_ref()),
            NodeId::Symbol { file, .. } => Some(file.as_ref()),
            NodeId::QueueJob { queue_file, .. } => Some(queue_file.as_ref()),
            NodeId::WorkflowJob { workflow_file, .. }
            | NodeId::WorkflowStep { workflow_file, .. } => Some(workflow_file.as_ref()),
            NodeId::Module(_) => None,
        }
    }
}
