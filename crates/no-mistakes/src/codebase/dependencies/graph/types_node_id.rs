use std::hash::{Hash, Hasher};

pub(crate) fn intern_node_path(path: impl AsRef<Path>) -> Arc<Path> {
    Arc::from(crate::codebase::ts_resolver::normalize_path(path.as_ref()))
}

pub(crate) fn intern_node_str(value: impl Into<Arc<str>>) -> Arc<str> {
    value.into()
}

fn interned_path_eq(left: &Arc<Path>, right: &Arc<Path>) -> bool {
    Arc::ptr_eq(left, right) || left.as_os_str() == right.as_os_str()
}

fn interned_str_eq(left: &Arc<str>, right: &Arc<str>) -> bool {
    Arc::ptr_eq(left, right) || left.as_ref() == right.as_ref()
}

fn hash_path<H: Hasher>(path: &Arc<Path>, state: &mut H) {
    path.as_os_str().hash(state);
}

fn hash_str<H: Hasher>(value: &Arc<str>, state: &mut H) {
    value.as_ref().hash(state);
}

impl PartialEq for NodeId {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::File(a), Self::File(b)) => interned_path_eq(a, b),
            (Self::Symbol { file: fa, symbol: sa }, Self::Symbol { file: fb, symbol: sb }) => {
                interned_path_eq(fa, fb) && interned_str_eq(sa, sb)
            }
            (Self::Module(a), Self::Module(b)) => interned_str_eq(a, b),
            (Self::QueueJob { queue_file: fa, job: ja }, Self::QueueJob { queue_file: fb, job: jb }) => {
                interned_path_eq(fa, fb) && interned_str_eq(ja, jb)
            }
            (
                Self::WorkflowJob {
                    workflow_file: fa,
                    job: ja,
                },
                Self::WorkflowJob {
                    workflow_file: fb,
                    job: jb,
                },
            ) => interned_path_eq(fa, fb) && interned_str_eq(ja, jb),
            (
                Self::WorkflowStep {
                    workflow_file: fa,
                    job: ja,
                    step: sa,
                },
                Self::WorkflowStep {
                    workflow_file: fb,
                    job: jb,
                    step: sb,
                },
            ) => interned_path_eq(fa, fb) && interned_str_eq(ja, jb) && sa == sb,
            _ => false,
        }
    }
}

impl Eq for NodeId {}

impl Hash for NodeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::File(path) => hash_path(path, state),
            Self::Symbol { file, symbol } => {
                hash_path(file, state);
                hash_str(symbol, state);
            }
            Self::Module(specifier) => hash_str(specifier, state),
            Self::QueueJob { queue_file, job } => {
                hash_path(queue_file, state);
                hash_str(job, state);
            }
            Self::WorkflowJob { workflow_file, job } => {
                hash_path(workflow_file, state);
                hash_str(job, state);
            }
            Self::WorkflowStep {
                workflow_file,
                job,
                step,
            } => {
                hash_path(workflow_file, state);
                hash_str(job, state);
                step.hash(state);
            }
        }
    }
}

impl NodeId {
    /// Construct a file node. Use in expressions only — match `NodeId::File(path)`.
    pub fn file(path: impl AsRef<Path>) -> Self {
        Self::File(intern_node_path(path))
    }

    /// Construct a symbol node. Use in expressions only — match `NodeId::Symbol { .. }`.
    pub fn symbol(path: impl AsRef<Path>, symbol: impl Into<Arc<str>>) -> Self {
        Self::Symbol {
            file: intern_node_path(path),
            symbol: intern_node_str(symbol),
        }
    }

    /// Construct a module node. Use in expressions only — match `NodeId::Module(...)`.
    pub fn module(value: impl Into<Arc<str>>) -> Self {
        Self::Module(intern_node_str(value))
    }

    /// Construct a queue-job node. Use in expressions only — match `NodeId::QueueJob { .. }`.
    pub fn queue_job(path: impl AsRef<Path>, job: impl Into<Arc<str>>) -> Self {
        Self::QueueJob {
            queue_file: intern_node_path(path),
            job: intern_node_str(job),
        }
    }

    /// Construct a workflow-job node. Use in expressions only — match `NodeId::WorkflowJob { .. }`.
    pub fn workflow_job(path: impl AsRef<Path>, job: impl Into<Arc<str>>) -> Self {
        Self::WorkflowJob {
            workflow_file: intern_node_path(path),
            job: intern_node_str(job),
        }
    }

    /// Construct a workflow-step node. Use in expressions only — match `NodeId::WorkflowStep { .. }`.
    pub fn workflow_step(path: impl AsRef<Path>, job: impl Into<Arc<str>>, step: usize) -> Self {
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
