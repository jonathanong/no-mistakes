fn local_caller_entries(
    context: &LocalCallerContext,
    target_symbols: &BTreeMap<PathBuf, BTreeSet<String>>,
    root: &Path,
    resolver: &dyn crate::codebase::ts_resolver::ImportResolution,
    test_filter: &TestFileFilter,
    want_tests: bool,
) -> Vec<CallerEntry> {
    let workspace = &context.workspace;
    let mut callers = BTreeMap::new();
    for (file, facts) in context.facts {
        let file = file.clone();
        let is_test = test_filter.is_match(root, &file);
        if !want_tests && !is_test && is_test_like_file(&file) {
            continue;
        }
        if is_test != want_tests {
            continue;
        }
        let symbols = facts
            .symbols
            .as_ref()
            .expect("imports_and_symbols fact plan collects symbols");
        let local_names = target_local_names(
            symbols,
            &file,
            target_symbols,
            resolver,
            workspace,
            context.remapper,
        );
        if local_names.is_empty() {
            continue;
        }
        let target_function_call_callers: BTreeSet<_> = facts
            .function_calls
            .iter()
            .filter(|call| matches_local_callee(&call.callee, &local_names))
            .filter_map(|call| call.caller.as_deref())
            .collect();
        for call in facts
            .function_calls
            .iter()
            .chain(facts.symbol_references.iter().filter(|call| {
                let Some(caller) = call.caller.as_deref() else {
                    return false;
                };
                !target_symbols.contains_key(&file)
                    || target_function_call_callers.contains(caller)
                    || !caller_is_target_export(symbols, &file, target_symbols, caller)
            }))
        {
            if !matches_local_callee(&call.callee, &local_names) {
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

struct CallerEntriesContext<'a> {
    root: &'a Path,
    test_filter: &'a TestFileFilter,
    export_nodes: &'a BTreeSet<NodeId>,
    file_target_symbols: &'a BTreeMap<String, BTreeSet<String>>,
    facts: &'a TsFactMap,
}

fn caller_entries(
    entries: &[NodeEntry],
    context: &CallerEntriesContext<'_>,
    want_tests: bool,
    extra_callers: &[CallerEntry],
) -> Vec<CallerEntry> {
    let mut by_key: BTreeMap<(String, Option<String>), CallerEntry> = BTreeMap::new();
    let mut dynamic_usage_cache: BTreeMap<String, bool> = BTreeMap::new();
    let export_files: BTreeSet<&Path> = context
        .export_nodes
        .iter()
        .filter_map(NodeId::as_file)
        .collect();
    let extra_file_callers: BTreeSet<&str> = extra_callers
        .iter()
        .filter(|caller| caller.symbol.is_none())
        .map(|caller| caller.file.as_str())
        .collect();
    for entry in entries {
        if context.export_nodes.contains(&entry.node) && !has_file_level_import_edge(&entry.via) {
            continue;
        }
        if let NodeId::File(file) = &entry.node {
            if export_files.contains(file.as_path()) && !has_file_level_import_edge(&entry.via) {
                continue;
            }
        }
        let Some((file, symbol)) = caller_parts(&entry.node, context.root) else {
            continue;
        };
        if matches!(entry.node, NodeId::File(_))
            && symbol.is_none()
            && !has_file_level_import_edge(&entry.via)
            && !extra_file_callers.contains(file.as_str())
        {
            continue;
        }
        if has_file_level_import_edge(&entry.via) {
            let Some(target_symbols) = context
                .file_target_symbols
                .get(file.as_str())
                .filter(|symbols| !symbols.is_empty())
            else {
                continue;
            };
            let uses_target = *dynamic_usage_cache.entry(file.clone()).or_insert_with(|| {
                file_entry_uses_any_symbol(
                    context.root,
                    file.as_str(),
                    target_symbols,
                    context.facts,
                )
            });
            if !uses_target {
                continue;
            }
        }
        let is_test = entry
            .node
            .as_file()
            .is_some_and(|path| context.test_filter.is_match(context.root, path));
        if !want_tests
            && !is_test
            && entry.node.as_file().is_some_and(is_test_like_file)
        {
            continue;
        }
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
