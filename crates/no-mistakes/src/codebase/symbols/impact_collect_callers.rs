fn local_caller_entries(
    graph: &DepGraph,
    target_file: &Path,
    target_symbol: &str,
    root: &Path,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
    test_filter: &TestFileFilter,
    want_tests: bool,
) -> Vec<CallerEntry> {
    let files: BTreeSet<PathBuf> = graph
        .all_files()
        .filter_map(NodeId::as_file)
        .map(Path::to_path_buf)
        .collect();
    let files: Vec<_> = files.into_iter().collect();
    let facts = crate::codebase::ts_source::facts::collect_ts_facts(
        &files,
        crate::codebase::ts_source::facts::TsFactPlan::imports_and_symbols(),
    );
    let mut callers = BTreeMap::new();
    for (file, facts) in facts {
        let is_test = test_filter.is_match(root, &file);
        if is_test != want_tests {
            continue;
        }
        let symbols = facts
            .symbols
            .as_ref()
            .expect("imports_and_symbols fact plan collects symbols");
        let local_names = target_local_names(symbols, &file, target_file, target_symbol, tsconfig);
        if local_names.is_empty() {
            continue;
        }
        for call in facts
            .function_calls
            .iter()
            .chain(facts.symbol_references.iter())
        {
            if !local_names.contains(&call.callee) {
                continue;
            }
            let symbol = call
                .caller
                .as_deref()
                .and_then(|caller| exported_symbol_for_local(symbols, caller));
            merge_caller_entry(
                &mut callers,
                CallerEntry {
                    file: relative_slash_path(root, &file),
                    symbol,
                    depth: 1,
                    via: vec!["symbol"],
                },
            );
        }
    }
    callers.into_values().collect()
}

fn target_local_names(
    symbols: &crate::codebase::ts_symbols::FileSymbols,
    file: &Path,
    target_file: &Path,
    target_symbol: &str,
    tsconfig: &crate::codebase::ts_resolver::TsConfig,
) -> BTreeSet<String> {
    if file == target_file {
        let mut names = BTreeSet::from([target_symbol.to_string()]);
        names.extend(symbols.exports.iter().filter_map(|export| {
            (!matches!(export.kind, ExportKind::ReExport { .. })
                && !export.is_type_only
                && export.name == target_symbol)
                .then(|| export.local.clone())
                .flatten()
        }));
        return names;
    }
    symbols
        .imports
        .iter()
        .filter_map(|import| {
            if import.is_type_only
                || resolve_import(&import.source, file, tsconfig)
                    .is_none_or(|resolved| resolved != target_file)
            {
                return None;
            }
            if import.imported == "*" {
                return Some(format!("{}.{}", import.local, target_symbol));
            }
            (import.imported == target_symbol).then(|| import.local.clone())
        })
        .collect()
}

fn exported_symbol_for_local(
    symbols: &crate::codebase::ts_symbols::FileSymbols,
    local: &str,
) -> Option<String> {
    symbols.exports.iter().find_map(|export| {
        if matches!(export.kind, ExportKind::ReExport { .. }) || export.is_type_only {
            return None;
        }
        (export.local.as_deref().unwrap_or(&export.name) == local).then(|| export.name.clone())
    })
}

fn caller_entries(
    entries: &[NodeEntry],
    root: &Path,
    test_filter: &TestFileFilter,
    want_tests: bool,
    export_nodes: &BTreeSet<NodeId>,
    extra_callers: &[CallerEntry],
) -> Vec<CallerEntry> {
    let mut by_key: BTreeMap<(String, Option<String>), CallerEntry> = BTreeMap::new();
    let export_files: BTreeSet<&Path> = export_nodes.iter().filter_map(NodeId::as_file).collect();
    for entry in entries {
        if export_nodes.contains(&entry.node) {
            continue;
        }
        if let NodeId::File(file) = &entry.node {
            if export_files.contains(file.as_path()) {
                continue;
            }
        }
        let Some((file, symbol)) = caller_parts(&entry.node, root) else {
            continue;
        };
        let is_test = entry
            .node
            .as_file()
            .is_some_and(|path| test_filter.is_match(root, path));
        if is_test != want_tests {
            continue;
        }
        insert_caller(&mut by_key, entry, file, symbol);
    }
    for caller in extra_callers {
        merge_caller_entry(&mut by_key, caller.clone());
    }
    let mut callers: Vec<_> = by_key.into_values().collect();
    callers.sort_by(|a, b| caller_sort_key(a).cmp(&caller_sort_key(b)));
    callers
}

fn merge_caller_entry(
    by_key: &mut BTreeMap<(String, Option<String>), CallerEntry>,
    caller: CallerEntry,
) {
    by_key
        .entry((caller.file.clone(), caller.symbol.clone()))
        .and_modify(|existing| {
            existing.depth = existing.depth.min(caller.depth);
            merge_via(&mut existing.via, &caller.via);
        })
        .or_insert(caller);
}

fn insert_caller(
    by_key: &mut BTreeMap<(String, Option<String>), CallerEntry>,
    entry: &NodeEntry,
    file: String,
    symbol: Option<String>,
) {
    let via = via_strings(&entry.via);
    by_key
        .entry((file.clone(), symbol.clone()))
        .and_modify(|existing| {
            existing.depth = existing.depth.min(entry.depth);
            merge_via(&mut existing.via, &via);
        })
        .or_insert(CallerEntry {
            file,
            symbol,
            depth: entry.depth,
            via,
        });
}

fn caller_sort_key(caller: &CallerEntry) -> (usize, &str, &str) {
    (
        caller.depth,
        caller.file.as_str(),
        caller.symbol.as_deref().unwrap_or_default(),
    )
}
