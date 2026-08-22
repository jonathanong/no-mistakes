impl NodeId {
    /// Construct a tRPC procedure node. Match `NodeId::TrpcProcedure { .. }`.
    pub fn trpc_procedure(path: impl AsRef<Path>, procedure: impl Into<Arc<str>>) -> Self {
        Self::TrpcProcedure {
            router_file: intern_node_path(path),
            procedure: intern_node_str(procedure),
        }
    }

    /// Session-interned tRPC procedure node. Match `NodeId::TrpcProcedure { .. }`.
    pub fn trpc_procedure_in(
        interner: &PathInterner,
        path: impl AsRef<Path>,
        procedure: impl AsRef<str>,
    ) -> Self {
        Self::TrpcProcedure {
            router_file: interner.intern_path(path),
            procedure: interner.intern_str(procedure),
        }
    }

    /// Path for any path-bearing variant, including queue/workflow/tRPC nodes.
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            NodeId::File(path) => Some(path.as_ref()),
            NodeId::Symbol { file, .. } => Some(file.as_ref()),
            NodeId::QueueJob { queue_file, .. } => Some(queue_file.as_ref()),
            NodeId::WorkflowJob { workflow_file, .. }
            | NodeId::WorkflowStep { workflow_file, .. } => Some(workflow_file.as_ref()),
            NodeId::TrpcProcedure { router_file, .. } => Some(router_file.as_ref()),
            NodeId::Module(_) => None,
        }
    }
}
