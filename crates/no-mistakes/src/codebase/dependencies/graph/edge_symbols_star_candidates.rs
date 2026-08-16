struct StarReexportCandidate {
    target: PathBuf,
    symbol: String,
    export_key: StarExportKey,
    kind: EdgeKind,
}

fn push_star_reexport_candidate_edges(
    inputs: &ExportEdgeInputs<'_>,
    candidate: StarReexportCandidate,
    edges: &mut Vec<Edge>,
) {
    edges.push((
        NodeId::file(inputs.path),
        NodeId::symbol(inputs.path, candidate.symbol.clone()),
        candidate.kind,
    ));
    edges.push((
        NodeId::symbol(inputs.path, candidate.symbol.clone()),
        NodeId::symbol(candidate.target, candidate.symbol),
        candidate.kind,
    ));
}
