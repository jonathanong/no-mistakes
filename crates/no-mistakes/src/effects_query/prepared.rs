pub(crate) fn selection_from_config(
    config: &crate::config::v2::NoMistakesConfig,
    kind: &str,
    categories: &[String],
) -> Result<EffectsSelection> {
    let Some(kind_config) = config.effects.get(kind) else {
        let available: Vec<&str> = config.effects.keys().map(String::as_str).collect();
        bail!(
            "unknown effects kind: {kind} (configured kinds: {})",
            if available.is_empty() {
                "<none>".to_string()
            } else {
                available.join(", ")
            }
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
    for target in &kind_config.targets {
        anyhow::ensure!(
            !target.module.trim().is_empty() && !target.export.trim().is_empty(),
            "effects kind `{kind}` targets require non-empty module and export"
        );
    }
    let mut targets: crate::fx::FxHashMap<
        String,
        crate::fx::FxHashMap<String, crate::config::v2::schema::EffectTargetConfig>,
    > = crate::fx::FxHashMap::default();
    for target in &kind_config.targets {
        if categories.is_empty()
            || target
                .category
                .as_ref()
                .is_some_and(|category| categories.contains(category))
        {
            targets
                .entry(target.module.clone())
                .or_default()
                .insert(target.export.clone(), target.clone());
        }
    }
    if names.is_empty() && targets.is_empty() {
        bail!("effects kind `{kind}` has no functions for the requested categories");
    }
    Ok(EffectsSelection {
        kind: kind.to_string(),
        names,
        targets,
    })
}

pub(crate) fn selection_fact_functions(
    selection: &EffectsSelection,
) -> impl Iterator<Item = String> + '_ {
    selection.names.keys().cloned()
}

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
    let mut call_sites = Vec::new();
    for (path, depth) in &file_depths {
        let Some(file) = facts.get(path) else {
            continue;
        };
        let relative_path = relative_slash_path(root, path);
        let mut matched_offsets = crate::fx::FxHashSet::default();
        for call in &file.effect_calls {
            if let Some(category) = selection.names.get(&call.callee) {
                call_sites.push(EffectCallSite {
                    file: relative_path.clone(),
                    line: call.line,
                    callee: call.callee.clone(),
                    category: category.clone(),
                    caller: call.caller.clone(),
                    depth: *depth,
                });
                matched_offsets.insert(call.offset);
            }
        }
        if selection.targets.is_empty() {
            continue;
        }
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
            let crate::codebase::dependencies::graph::ResolvedCallTarget::ModuleExport {
                specifier,
                export_path,
                ..
            } = &site.target
            else {
                continue;
            };
            let Some(target) = selection
                .targets
                .get(specifier.as_str())
                .and_then(|exports| exports.get(export_path.as_str()))
            else {
                continue;
            };
            if !matched_offsets.insert(site.offset) {
                continue;
            }
            call_sites.push(EffectCallSite {
                file: relative_path.clone(),
                line: site.line as usize,
                callee: target.export.clone(),
                category: target.category.clone(),
                caller: callers
                    .get(&site.offset)
                    .and_then(|caller| (*caller).clone()),
                depth: *depth,
            });
        }
    }
    call_sites.sort();
    let mut by_category: BTreeMap<String, usize> = BTreeMap::new();
    for site in &call_sites {
        let label = site
            .category
            .clone()
            .unwrap_or_else(|| "uncategorized".to_string());
        *by_category.entry(label).or_insert(0) += 1;
    }
    Ok(EffectsReport {
        kind: selection.kind.clone(),
        entry: relative_slash_path(root, &entry_abs),
        call_sites,
        by_category,
    })
}
