fn extend_scoped_seed_diagnostics(
    runtime_diagnostics: &mut Vec<crate::codebase::ts_resolver::TsConfigDiagnostic>,
    seed: &[crate::codebase::ts_resolver::TsConfigDiagnostic],
    entries: &[graph::NodeEntry],
) {
    let reachable = entries
        .iter()
        .filter_map(|entry| entry.node.as_file())
        .collect::<std::collections::HashSet<_>>();
    for diagnostic in seed {
        if runtime_diagnostics.contains(diagnostic) {
            continue;
        }
        let Some(file) = diagnostic.file.as_deref() else {
            runtime_diagnostics.push(diagnostic.clone());
            continue;
        };
        if reachable.contains(file) {
            runtime_diagnostics.push(diagnostic.clone());
        }
    }
}
