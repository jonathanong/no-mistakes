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
    let interner = inputs.interner;
    edges.push((
        NodeId::file_in(interner, inputs.path),
        NodeId::symbol_in(interner, inputs.path, candidate.symbol.clone()),
        candidate.kind,
    ));
    edges.push((
        NodeId::symbol_in(interner, inputs.path, candidate.symbol.clone()),
        NodeId::symbol_in(interner, candidate.target, candidate.symbol),
        candidate.kind,
    ));
}
