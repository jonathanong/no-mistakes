pub(crate) fn run_with_prepared(
    root: &Path,
    selection: &EffectsSelection,
    entry: &Path,
    depth: Option<usize>,
    graph: &DepGraph,
    facts: &crate::codebase::ts_source::facts::TsFactMap,
    interner: &crate::codebase::analysis_session::PathInterner,
) -> Result<EffectsReport> {
    let entry_abs = if entry.is_absolute() {
        entry.to_path_buf()
    } else {
        root.join(entry)
    };
    if !entry_abs.is_file() {
        bail!("entry file not found: {}", entry_abs.display());
    }
    let entry_node = NodeId::file_in(interner, normalize_path(&entry_abs));
    let allowed = runtime_edges();
    let reachable = graph.deps_of(std::slice::from_ref(&entry_node), depth, Some(&allowed));
    let mut file_depths: HashMap<PathBuf, usize> = HashMap::new();
    if let NodeId::File(path) = &entry_node {
        file_depths.insert(path.to_path_buf(), 0);
    }
    for entry in &reachable {
        if let NodeId::File(path) = &entry.node {
            file_depths.entry(path.to_path_buf()).or_insert(entry.depth);
        }
    }
    let target_nodes: crate::fx::FxHashMap<_, _> = selection
        .targets
        .iter()
        .flat_map(|(module, exports)| {
            exports.values().flat_map(|target| {
                graph
                    .module_export_nodes(module, &target.export)
                    .into_iter()
                    .map(move |node| (node, target))
            })
        })
        .collect();
    let mut call_sites = Vec::new();
    for (path, depth) in &file_depths {
        let Some(file) = facts.get(path) else {
            continue;
        };
        let relative_path = relative_slash_path(root, path);
        let mut matches = crate::fx::FxHashMap::<u32, EffectCallSite>::default();
        for call in &file.effect_calls {
            if let Some(category) = selection.names.get(&call.callee) {
                matches
                    .entry(call.offset)
                    .or_insert_with(|| EffectCallSite {
                        file: relative_path.clone(),
                        line: call.line,
                        callee: call.callee.clone(),
                        category: category.clone(),
                        caller: call.caller.clone(),
                        depth: *depth,
                        alternatives: None,
                    });
            }
        }
        if needs_resolved_effects(selection) {
            let callers: crate::fx::FxHashMap<_, _> = file
                .function_calls
                .iter()
                .map(|call| (call.offset, &call.syntactic_caller))
                .collect();
            for site in graph.call_sites_in_file(path) {
                if !matches!(
                    site.invocation,
                    crate::codebase::dependencies::extract::InvocationKind::Call
                        | crate::codebase::dependencies::extract::InvocationKind::Construct
                ) {
                    continue;
                }
                let multiple = has_effect_alternatives(site);
                // Keep the established spelling/config precedence for singleton identities.
                if !multiple && matches.contains_key(&site.offset) {
                    continue;
                }
                let mut record = |matched: EffectCallAlternative| match matches.entry(site.offset) {
                    std::collections::hash_map::Entry::Occupied(mut entry) => {
                        entry.get_mut().add_match(matched)
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(EffectCallSite {
                            file: relative_path.clone(),
                            line: site.line as usize,
                            callee: matched.callee,
                            category: matched.category,
                            caller: callers
                                .get(&site.offset)
                                .and_then(|caller| (*caller).clone()),
                            depth: *depth,
                            alternatives: None,
                        });
                    }
                };
                visit_effect_matches(selection, site, &target_nodes, &mut record);
            }
        }
        for mut site in matches.into_values() {
            site.normalize_matches();
            call_sites.push(site);
        }
    }
    call_sites.sort();
    let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
    for site in &call_sites {
        site.count_categories(&mut by_category);
    }
    Ok(EffectsReport {
        kind: selection.kind.clone(),
        entry: relative_slash_path(root, &entry_abs),
        call_sites,
        by_category,
    })
}
