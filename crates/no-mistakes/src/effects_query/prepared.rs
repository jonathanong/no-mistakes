pub(crate) fn selection_from_config(
    config: &crate::config::v2::NoMistakesConfig,
    kind: &str,
    categories: &[String],
) -> Result<EffectsSelection> {
    let Some(kind_config) = config.effects.get(kind) else {
        let available: Vec<&str> = config.effects.keys().map(String::as_str).collect();
        bail!(
            "unknown effects kind: {kind} (configured kinds: {})",
            if available.is_empty() { "<none>".to_string() } else { available.join(", ") }
        );
    };
    let mut names = HashMap::new();
    for (category, functions) in &kind_config.categories {
        if !categories.is_empty() && !categories.iter().any(|value| value == category) {
            continue;
        }
        for function in functions {
            names.insert(function.clone(), Some(category.clone()));
        }
    }
    if categories.is_empty() {
        for function in &kind_config.functions {
            names.entry(function.clone()).or_insert(None);
        }
    }
    if names.is_empty() {
        bail!("effects kind `{kind}` has no functions for the requested categories");
    }
    Ok(EffectsSelection { kind: kind.to_string(), names })
}

pub(crate) fn selection_fact_functions(selection: &EffectsSelection) -> impl Iterator<Item = String> + '_ {
    selection.names.keys().cloned()
}

pub(crate) fn run_with_prepared(
    root: &Path,
    selection: &EffectsSelection,
    entry: &Path,
    depth: Option<usize>,
    graph: &DepGraph,
    facts: &crate::codebase::ts_source::facts::TsFactMap,
) -> Result<EffectsReport> {
    let entry_abs = if entry.is_absolute() { entry.to_path_buf() } else { root.join(entry) };
    if !entry_abs.is_file() {
        bail!("entry file not found: {}", entry_abs.display());
    }
    let entry_node = NodeId::File(normalize_path(&entry_abs));
    let allowed = runtime_edges();
    let reachable = graph.deps_of(std::slice::from_ref(&entry_node), depth, Some(&allowed));
    let mut file_depths: HashMap<PathBuf, usize> = HashMap::new();
    if let NodeId::File(path) = &entry_node {
        file_depths.insert(path.clone(), 0);
    }
    for entry in &reachable {
        if let NodeId::File(path) = &entry.node {
            file_depths.entry(path.clone()).or_insert(entry.depth);
        }
    }
    let mut call_sites: Vec<EffectCallSite> = file_depths
        .iter()
        .filter_map(|(path, depth)| facts.get(path).map(|file| (path, file, *depth)))
        .flat_map(|(path, file, depth)| {
            let relative_path = relative_slash_path(root, path);
            file.effect_calls.iter().filter_map(move |call| {
                selection.names.get(&call.callee).map(|category| EffectCallSite {
                    file: relative_path.clone(),
                    line: call.line,
                    callee: call.callee.clone(),
                    category: category.clone(),
                    caller: call.caller.clone(),
                    depth,
                })
            })
        })
        .collect();
    call_sites.sort();
    let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
    for site in &call_sites {
        let label = site.category.clone().unwrap_or_else(|| "uncategorized".to_string());
        *by_category.entry(label).or_insert(0) += 1;
    }
    Ok(EffectsReport {
        kind: selection.kind.clone(),
        entry: relative_slash_path(root, &entry_abs),
        call_sites,
        by_category,
    })
}
