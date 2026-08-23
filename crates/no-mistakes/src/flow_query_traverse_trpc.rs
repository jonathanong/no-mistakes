fn trpc_procedure_flow_node(
    id: String,
    depth: usize,
    root: &Path,
    router_file: &Path,
    procedure: &str,
) -> FlowNode {
    FlowNode {
        id,
        kind: "trpc-procedure",
        depth,
        file: None,
        symbol: None,
        module: None,
        queue_file: None,
        job: None,
        workflow_file: None,
        step: None,
        router_file: Some(relative(root, router_file)),
        procedure: Some(procedure.to_string()),
    }
}
