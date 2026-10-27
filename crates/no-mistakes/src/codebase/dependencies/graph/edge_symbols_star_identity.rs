/// Keep one star binding when every path reaches the same export.
///
/// Two `export *` paths often name one function: a direct star and a named
/// re-export of that star. Those are one symbol. Distinct definitions stay
/// ambiguous and produce no binding.
fn agreed_star_reexport_candidates(
    inputs: &ExportEdgeInputs<'_>,
    candidates: Vec<StarReexportCandidate>,
) -> Vec<StarReexportCandidate> {
    let mut order = Vec::new();
    let mut grouped: HashMap<StarExportKey, Vec<StarReexportCandidate>> = HashMap::new();
    for candidate in candidates {
        grouped
            .entry(candidate.export_key.clone())
            .or_insert_with(|| {
                order.push(candidate.export_key.clone());
                Vec::new()
            })
            .push(candidate);
    }
    let mut kept = Vec::new();
    for key in order {
        let group = grouped
            .remove(&key)
            .expect("star group key is inserted before lookup");
        if group.len() == 1 || star_group_shares_origin(inputs, &key, &group) {
            kept.extend(group.into_iter().take(1));
        }
    }
    kept
}

fn star_group_shares_origin(
    inputs: &ExportEdgeInputs<'_>,
    key: &StarExportKey,
    group: &[StarReexportCandidate],
) -> bool {
    let mut origins = group.iter().map(|candidate| {
        let mut seen = HashSet::new();
        star_export_origin(
            inputs,
            &candidate.target,
            &key.name,
            &key.namespace,
            &mut seen,
        )
    });
    let Some(first) = origins.next().flatten() else {
        return false;
    };
    origins.all(|origin| origin.as_ref() == Some(&first))
}

fn star_export_origin(
    inputs: &ExportEdgeInputs<'_>,
    file: &Path,
    name: &str,
    namespace: &StarExportNamespace,
    seen: &mut HashSet<(PathBuf, String)>,
) -> Option<(PathBuf, String)> {
    if !seen.insert((file.to_path_buf(), name.to_owned())) {
        return None;
    }
    let symbols = inputs
        .facts
        .get_ts_facts(file)
        .and_then(|facts| facts.symbols.as_ref())?;
    if let Some(export) = concrete_named_export(symbols, name, namespace) {
        if let ExportKind::ReExport { source, imported } = &export.kind {
            if imported != "*" {
                let (next, _) = resolve_star_source(inputs, file, source, export.is_type_only)?;
                return star_export_origin(inputs, &next, imported, namespace, seen);
            }
        }
        return Some((file.to_path_buf(), name.to_owned()));
    }
    origin_through_stars(inputs, file, symbols, name, namespace, seen)
}

fn concrete_named_export<'a>(
    symbols: &'a crate::codebase::ts_symbols::FileSymbols,
    name: &str,
    namespace: &StarExportNamespace,
) -> Option<&'a crate::codebase::ts_symbols::Export> {
    let matches: Vec<_> = symbols
        .exports
        .iter()
        .filter(|export| export.name != "*" && export_symbol_name(export) == name)
        .collect();
    matches
        .iter()
        .copied()
        .find(|export| &star_export_key(export, false).namespace == namespace)
        .or_else(|| matches.first().copied())
}

fn origin_through_stars(
    inputs: &ExportEdgeInputs<'_>,
    file: &Path,
    symbols: &crate::codebase::ts_symbols::FileSymbols,
    name: &str,
    namespace: &StarExportNamespace,
    seen: &mut HashSet<(PathBuf, String)>,
) -> Option<(PathBuf, String)> {
    let mut found = None;
    for export in &symbols.exports {
        let ExportKind::ReExport { source, imported } = &export.kind else {
            continue;
        };
        if imported != "*" || export.name != "*" {
            continue;
        }
        let Some((next, _)) = resolve_star_source(inputs, file, source, export.is_type_only) else {
            continue;
        };
        let Some(origin) = star_export_origin(inputs, &next, name, namespace, seen) else {
            continue;
        };
        if found.as_ref().is_some_and(|previous| previous != &origin) {
            return None;
        }
        found = Some(origin);
    }
    found
}
