fn collect_trpc_edges(
    root: &Path,
    graph_files: &GraphFiles,
    facts: Option<&dyn TsFactLookup>,
    config_options: Option<&GraphConfigOptions>,
    interner: &PathInterner,
) -> Vec<Edge> {
    let Some(options) = config_options else {
        return Vec::new();
    };
    if options.trpc_routers.is_empty() {
        return Vec::new();
    }
    let Some(globset) = compile_trpc_router_globset(&options.trpc_routers) else {
        return Vec::new();
    };
    let Some(facts) = facts else {
        return Vec::new();
    };
    let mut procedures: HashMap<String, PathBuf> = HashMap::new();
    for path in graph_files.indexable() {
        let rel = path.strip_prefix(root).unwrap_or(path);
        if !globset.is_match(rel) {
            continue;
        }
        let Some(file_facts) = facts.get_ts_facts(path) else {
            continue;
        };
        for procedure in &file_facts.trpc_procedures {
            procedures
                .entry(procedure.clone())
                .or_insert_with(|| path.clone());
        }
    }
    let mut edges = Vec::new();
    for path in graph_files.indexable() {
        let Some(file_facts) = facts.get_ts_facts(path) else {
            continue;
        };
        for call in &file_facts.trpc_calls {
            let Some(router_file) = procedures.get(&call.path) else {
                continue;
            };
            let procedure = NodeId::trpc_procedure_in(interner, router_file, call.path.clone());
            edges.push((
                NodeId::file_in(interner, path),
                procedure.clone(),
                EdgeKind::TrpcCall,
            ));
            edges.push((
                procedure,
                NodeId::file_in(interner, router_file),
                EdgeKind::TrpcProcedure,
            ));
        }
    }
    edges.sort_by(|left, right| {
        (
            left.0.display_name(root),
            left.1.display_name(root),
            left.2.sort_key(),
        )
            .cmp(&(
                right.0.display_name(root),
                right.1.display_name(root),
                right.2.sort_key(),
            ))
    });
    edges.dedup();
    edges
}

fn collect_trpc_edges_for_plan(
    edge_inputs: &GraphEdgeBuildInputs<'_>,
    facts: Option<&dyn TsFactLookup>,
) -> Vec<Edge> {
    if !edge_inputs.plan.trpc {
        return Vec::new();
    }
    collect_trpc_edges(
        edge_inputs.root,
        edge_inputs.graph_files,
        facts,
        edge_inputs.config_options,
        &edge_inputs.interner,
    )
}
