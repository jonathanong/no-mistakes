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
        NodeId::File(inputs.path.to_path_buf()),
        NodeId::Symbol {
            file: inputs.path.to_path_buf(),
            symbol: candidate.symbol.clone(),
        },
        candidate.kind,
    ));
    edges.push((
        NodeId::Symbol {
            file: inputs.path.to_path_buf(),
            symbol: candidate.symbol.clone(),
        },
        NodeId::Symbol {
            file: candidate.target,
            symbol: candidate.symbol,
        },
        candidate.kind,
    ));
}
