pub(crate) fn intern_node_path(path: impl AsRef<Path>) -> Arc<Path> {
    Arc::from(crate::codebase::ts_resolver::normalize_path(path.as_ref()))
}

pub(crate) fn intern_node_str(value: impl Into<Arc<str>>) -> Arc<str> {
    value.into()
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
            ) => fa == fb && sa == sb,
            (Self::Module(a), Self::Module(b)) => a == b,
            (
                Self::QueueJob {
                    queue_file: fa,
                    job: ja,
                },
                Self::QueueJob {
                    queue_file: fb,
                    job: jb,
                },
            ) => fa == fb && ja == jb,
            (
                Self::WorkflowJob {
                    workflow_file: fa,
                    job: ja,
                },
                Self::WorkflowJob {
                    workflow_file: fb,
                    job: jb,
                },
            ) => fa == fb && ja == jb,
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
            ) => fa == fb && ja == jb && sa == sb,
            (
                Self::TrpcProcedure {
                    router_file: fa,
                    procedure: pa,
                },
                Self::TrpcProcedure {
                    router_file: fb,
                    procedure: pb,
                },
            ) => fa == fb && pa == pb,
            _ => false,
        }
    }
}

impl Eq for NodeId {}

impl Hash for NodeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::File(path) => path.hash(state),
            Self::Symbol { file, symbol } => {
                file.hash(state);
                symbol.hash(state);
            }
            Self::Module(specifier) => specifier.hash(state),
            Self::QueueJob { queue_file, job } => {
                queue_file.hash(state);
                job.hash(state);
            }
            Self::WorkflowJob { workflow_file, job } => {
                workflow_file.hash(state);
                job.hash(state);
            }
            Self::WorkflowStep {
                workflow_file,
                job,
                step,
            } => {
                workflow_file.hash(state);
                job.hash(state);
                step.hash(state);
            }
            Self::TrpcProcedure {
                router_file,
                procedure,
            } => {
                router_file.hash(state);
                procedure.hash(state);
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
    pub fn symbol(path: impl AsRef<Path>, symbol: impl Into<InternedStr>) -> Self {
        Self::Symbol {
            file: FileNode::new(intern_node_path(path)),
            symbol: symbol.into(),
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
            symbol: InternedStr::new(interner.intern_str(symbol)),
        }
    }

    /// Construct a module node. Use in expressions only — match `NodeId::Module(...)`.
    pub fn module(value: impl Into<InternedStr>) -> Self {
        Self::Module(value.into())
    }

    /// Session-interned module node. Match `NodeId::Module(...)`.
    pub fn module_in(interner: &PathInterner, value: impl AsRef<str>) -> Self {
        Self::Module(InternedStr::new(interner.intern_str(value)))
    }

    /// Construct a queue-job node. Use in expressions only — match `NodeId::QueueJob { .. }`.
    pub fn queue_job(path: impl AsRef<Path>, job: impl Into<InternedStr>) -> Self {
        Self::QueueJob {
            queue_file: FileNode::new(intern_node_path(path)),
            job: job.into(),
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
            job: InternedStr::new(interner.intern_str(job)),
        }
    }

    /// Construct a workflow-job node. Use in expressions only — match `NodeId::WorkflowJob { .. }`.
    pub fn workflow_job(path: impl AsRef<Path>, job: impl Into<InternedStr>) -> Self {
        Self::WorkflowJob {
            workflow_file: FileNode::new(intern_node_path(path)),
            job: job.into(),
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
            job: InternedStr::new(interner.intern_str(job)),
        }
    }

    /// Construct a workflow-step node. Use in expressions only — match `NodeId::WorkflowStep { .. }`.
    pub fn workflow_step(path: impl AsRef<Path>, job: impl Into<InternedStr>, step: usize) -> Self {
        Self::WorkflowStep {
            workflow_file: FileNode::new(intern_node_path(path)),
            job: job.into(),
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
            job: InternedStr::new(interner.intern_str(job)),
            step,
        }
    }
}

include!("types_node_id_trpc.rs");
