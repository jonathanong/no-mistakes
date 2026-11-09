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

fn ordinary_diagnostic_visible(
    root: &Path,
    diagnostic: &crate::codebase::ts_resolver::TsConfigDiagnostic,
    excluded: &std::collections::HashSet<PathBuf>,
) -> bool {
    if excluded.is_empty() {
        return true;
    }
    diagnostic.file.as_deref().is_none_or(|file| {
        let absolute = if crate::codebase::ts_source::is_portably_absolute_path(file) {
            file.to_path_buf()
        } else {
            root.join(file)
        };
        let normalized = crate::codebase::ts_resolver::normalize_path(&absolute);
        if excluded.contains(&normalized) {
            return false;
        }
        let root_text = root.to_string_lossy();
        let bytes = root_text.as_bytes();
        let windows_root = root_text.starts_with(r"\\")
            || root_text.starts_with("//")
            || (bytes.len() >= 3
                && bytes[0].is_ascii_alphabetic()
                && bytes[1] == b':'
                && (bytes[2] == b'/' || bytes[2] == b'\\'));
        if !windows_root {
            return true;
        }
        let relative = crate::codebase::ts_source::relative_slash_path(root, &absolute);
        !excluded
            .iter()
            .any(|path| crate::codebase::ts_source::relative_slash_path(root, path) == relative)
    })
}

fn ordinary_report_diagnostics(
    shared: &SharedTraversalContext,
    runtime_diagnostics: Vec<crate::codebase::ts_resolver::TsConfigDiagnostic>,
    ordinary_excluded: &std::collections::HashSet<PathBuf>,
) -> Vec<crate::codebase::ts_resolver::TsConfigDiagnostic> {
    let diagnostics: Vec<_> = shared
        .tsconfig_build_diagnostics
        .iter()
        .cloned()
        .chain(runtime_diagnostics)
        .filter(|diagnostic| {
            ordinary_diagnostic_visible(&shared.root, diagnostic, ordinary_excluded)
        })
        .map(|mut diagnostic| {
            let root_text = shared.root.to_string_lossy();
            diagnostic.detail = diagnostic.detail.replace(&format!("{root_text}/"), "");
            diagnostic.config = diagnostic.config.map(|config| {
                config
                    .strip_prefix(&shared.root)
                    .unwrap_or(&config)
                    .to_path_buf()
            });
            diagnostic.file = diagnostic.file.map(|file| {
                file.strip_prefix(&shared.root)
                    .unwrap_or(&file)
                    .to_path_buf()
            });
            diagnostic.candidates = diagnostic
                .candidates
                .into_iter()
                .map(|candidate| {
                    candidate
                        .strip_prefix(&shared.root)
                        .unwrap_or(&candidate)
                        .to_path_buf()
                })
                .collect();
            diagnostic
        })
        .collect();
    if shared.ordinary_excluded_files.is_empty() {
        return diagnostics;
    }
    let keep = {
        let mut seen = std::collections::BTreeSet::new();
        diagnostics
            .iter()
            .map(|diagnostic| {
                ordinary_diagnostic_visible(
                    &shared.root,
                    diagnostic,
                    &shared.ordinary_excluded_files,
                ) || seen.insert(diagnostic)
            })
            .collect::<Vec<_>>()
    };
    diagnostics
        .into_iter()
        .zip(keep)
        .filter_map(|(diagnostic, keep)| keep.then_some(diagnostic))
        .collect()
}
