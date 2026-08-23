pub(crate) fn intern_node_path(path: impl AsRef<Path>) -> Arc<Path> {
    Arc::from(crate::codebase::ts_resolver::normalize_path(path.as_ref()))
}

pub(crate) fn intern_node_str(value: impl Into<Arc<str>>) -> Arc<str> {
    value.into()
}

fn interned_str_eq(left: &Arc<str>, right: &Arc<str>) -> bool {
    Arc::ptr_eq(left, right) || left.as_ref() == right.as_ref()
}

fn hash_file<H: Hasher>(file: &FileNode, state: &mut H) {
    file.hash(state);
}

fn hash_str<H: Hasher>(value: &Arc<str>, state: &mut H) {
    value.as_ref().hash(state);
}

impl PartialEq for NodeId {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::File(a), Self::File(b)) => a == b,
            (
                Self::Symbol {
                    file: fa,
                    symbol: sa,
                },
                Self::Symbol {
                    file: fb,
                    symbol: sb,
                },
            ) => fa == fb && interned_str_eq(sa, sb),
            (Self::Module(a), Self::Module(b)) => interned_str_eq(a, b),
            (
                Self::QueueJob {
                    queue_file: fa,
                    job: ja,
                },
                Self::QueueJob {
                    queue_file: fb,
                    job: jb,
                },
            ) => fa == fb && interned_str_eq(ja, jb),
            (
                Self::WorkflowJob {
                    workflow_file: fa,
                    job: ja,
                },
                Self::WorkflowJob {
                    workflow_file: fb,
                    job: jb,
                },
            ) => fa == fb && interned_str_eq(ja, jb),
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
            ) => fa == fb && interned_str_eq(ja, jb) && sa == sb,
            (
                Self::TrpcProcedure {
                    router_file: fa,
                    procedure: pa,
                },
                Self::TrpcProcedure {
                    router_file: fb,
                    procedure: pb,
                },
            ) => fa == fb && interned_str_eq(pa, pb),
            _ => false,
        }
    }
}

impl Eq for NodeId {}

impl Hash for NodeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::File(path) => hash_file(path, state),
            Self::Symbol { file, symbol } => {
                hash_file(file, state);
                hash_str(symbol, state);
            }
            Self::Module(specifier) => hash_str(specifier, state),
            Self::QueueJob { queue_file, job } => {
                hash_file(queue_file, state);
                hash_str(job, state);
            }
            Self::WorkflowJob { workflow_file, job } => {
                hash_file(workflow_file, state);
                hash_str(job, state);
            }
            Self::WorkflowStep {
                workflow_file,
                job,
                step,
            } => {
                hash_file(workflow_file, state);
                hash_str(job, state);
                step.hash(state);
            }
            Self::TrpcProcedure {
                router_file,
                procedure,
            } => {
                hash_file(router_file, state);
                hash_str(procedure, state);
            }
        }
    }
}

impl NodeId {
    /// Construct a file node. Use in expressions only — match `NodeId::File(path)`.
    pub fn file(path: impl AsRef<Path>) -> Self {
        Self::File(FileNode::new(intern_node_path(path)))
    }

    /// Session-interned file node. Match `NodeId::File(path)`.
    pub fn file_in(interner: &PathInterner, path: impl AsRef<Path>) -> Self {
        Self::File(FileNode::new(interner.intern_path(path)))
    }

    /// Construct a symbol node. Use in expressions only — match `NodeId::Symbol { .. }`.
    pub fn symbol(path: impl AsRef<Path>, symbol: impl Into<Arc<str>>) -> Self {
        Self::Symbol {
            file: FileNode::new(intern_node_path(path)),
            symbol: intern_node_str(symbol),
        }
    }

    /// Session-interned symbol node. Match `NodeId::Symbol { .. }`.
    pub fn symbol_in(
        interner: &PathInterner,
        path: impl AsRef<Path>,
        symbol: impl AsRef<str>,
    ) -> Self {
        Self::Symbol {
            file: FileNode::new(interner.intern_path(path)),
            symbol: interner.intern_str(symbol),
        }
    }

    /// Construct a module node. Use in expressions only — match `NodeId::Module(...)`.
    pub fn module(value: impl Into<Arc<str>>) -> Self {
        Self::Module(intern_node_str(value))
    }

    /// Session-interned module node. Match `NodeId::Module(...)`.
    pub fn module_in(interner: &PathInterner, value: impl AsRef<str>) -> Self {
        Self::Module(interner.intern_str(value))
    }

    /// Construct a queue-job node. Use in expressions only — match `NodeId::QueueJob { .. }`.
    pub fn queue_job(path: impl AsRef<Path>, job: impl Into<Arc<str>>) -> Self {
        Self::QueueJob {
            queue_file: FileNode::new(intern_node_path(path)),
            job: intern_node_str(job),
        }
    }

    /// Session-interned queue-job node. Match `NodeId::QueueJob { .. }`.
    pub fn queue_job_in(
        interner: &PathInterner,
        path: impl AsRef<Path>,
        job: impl AsRef<str>,
    ) -> Self {
        Self::QueueJob {
            queue_file: FileNode::new(interner.intern_path(path)),
            job: interner.intern_str(job),
        }
    }

    /// Construct a workflow-job node. Use in expressions only — match `NodeId::WorkflowJob { .. }`.
    pub fn workflow_job(path: impl AsRef<Path>, job: impl Into<Arc<str>>) -> Self {
        Self::WorkflowJob {
            workflow_file: FileNode::new(intern_node_path(path)),
            job: intern_node_str(job),
        }
    }

    /// Session-interned workflow-job node. Match `NodeId::WorkflowJob { .. }`.
    pub fn workflow_job_in(
        interner: &PathInterner,
        path: impl AsRef<Path>,
        job: impl AsRef<str>,
    ) -> Self {
        Self::WorkflowJob {
            workflow_file: FileNode::new(interner.intern_path(path)),
            job: interner.intern_str(job),
        }
    }

    /// Construct a workflow-step node. Use in expressions only — match `NodeId::WorkflowStep { .. }`.
    pub fn workflow_step(path: impl AsRef<Path>, job: impl Into<Arc<str>>, step: usize) -> Self {
        Self::WorkflowStep {
            workflow_file: FileNode::new(intern_node_path(path)),
            job: intern_node_str(job),
            step,
        }
    }

    /// Session-interned workflow-step node. Match `NodeId::WorkflowStep { .. }`.
    pub fn workflow_step_in(
        interner: &PathInterner,
        path: impl AsRef<Path>,
        job: impl AsRef<str>,
        step: usize,
    ) -> Self {
        Self::WorkflowStep {
            workflow_file: FileNode::new(interner.intern_path(path)),
            job: interner.intern_str(job),
            step,
        }
    }
}

include!("types_node_id_trpc.rs");
