fn extend_scoped_seed_diagnostics(
    runtime_diagnostics: &mut Vec<crate::codebase::ts_resolver::TsConfigDiagnostic>,
    seed: &[crate::codebase::ts_resolver::TsConfigDiagnostic],
    entries: &[graph::NodeEntry],
) {
    let reachable = entries
        .iter()
        .filter_map(|entry| entry.node.as_file())
        .collect::<Vec<_>>();
    for diagnostic in seed {
        if runtime_diagnostics.contains(diagnostic) {
            continue;
        }
        let Some(file) = diagnostic.file.as_deref() else {
            runtime_diagnostics.push(diagnostic.clone());
            continue;
        };
        if reachable.iter().any(|path| same_source_file(path, file)) {
            runtime_diagnostics.push(diagnostic.clone());
        }
    }
}

fn same_source_file(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    let left_norm = crate::codebase::ts_resolver::normalize_path(left);
    let right_norm = crate::codebase::ts_resolver::normalize_path(right);
    if left_norm == right_norm {
        return true;
    }
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => {
            crate::codebase::ts_resolver::normalize_path(&left)
                == crate::codebase::ts_resolver::normalize_path(&right)
        }
        _ => false,
    }
}
