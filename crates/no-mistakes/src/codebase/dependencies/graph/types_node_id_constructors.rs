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
            callable_id: None,
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
            callable_id: None,
        }
    }

    pub fn callable_in(
        interner: &PathInterner,
        path: impl AsRef<Path>,
        symbol: impl AsRef<str>,
        callable_id: crate::codebase::dependencies::extract::CallableId,
    ) -> Self {
        Self::Symbol {
            file: FileNode::new(interner.intern_path(path)),
            symbol: InternedStr::new(interner.intern_str(symbol)),
            callable_id: Some(callable_id),
        }
    }

    pub fn callable(
        path: impl AsRef<Path>,
        symbol: impl Into<InternedStr>,
        callable_id: crate::codebase::dependencies::extract::CallableId,
    ) -> Self {
        Self::Symbol {
            file: FileNode::new(intern_node_path(path)),
            symbol: symbol.into(),
            callable_id: Some(callable_id),
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
