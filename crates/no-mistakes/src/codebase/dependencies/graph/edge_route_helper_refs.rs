fn route_pattern_should_skip(
    route_pattern: &str,
    backend_prefixes: &[String],
    backend_exact: &[String],
    has_backend_filter: bool,
    frontend_root_empty: bool,
) -> bool {
    let is_backend = backend_prefixes
        .iter()
        .any(|prefix| route_pattern.starts_with(prefix.as_str()))
        || backend_exact.iter().any(|exact| exact == route_pattern);
    has_backend_filter && !is_backend && frontend_root_empty
}

fn push_matching_route_edges(
    edges: &mut Vec<Edge>,
    source: &Path,
    route_pattern: &str,
    all_patterns: &[String],
    pattern_to_files: &HashMap<String, Vec<PathBuf>>,
) {
    use crate::codebase::ts_routes::matcher;

    for pattern in all_patterns {
        if matcher::matches(route_pattern, pattern) {
            if let Some(def_files) = pattern_to_files.get(pattern) {
                for def_file in def_files.iter().filter(|def_file| *def_file != source) {
                    push_route_ref_edge(edges, source, def_file);
                }
            }
        }
    }
}

fn route_helper_ref_patterns(
    path: &Path,
    file_facts: &crate::codebase::ts_source::facts::TsFileFacts,
    facts: &dyn TsFactLookup,
    resolver: &crate::codebase::ts_resolver::ImportResolver<'_>,
) -> Vec<String> {
    let mut patterns = Vec::new();
    for helper_ref in &file_facts.route_helper_refs {
        let mut helper_patterns = local_route_helper_patterns(
            &helper_ref.callee,
            &file_facts.route_helpers,
        );
        helper_patterns.extend(imported_route_helper_patterns(
            path,
            &helper_ref.callee,
            &file_facts.route_helper_imports,
            facts,
            resolver,
        ));
        patterns.extend(route_helper_ref_wrapped_patterns(helper_ref, helper_patterns));
    }
    patterns.sort();
    patterns.dedup();
    patterns
}

fn local_route_helper_patterns(
    callee: &str,
    helpers: &[crate::codebase::ts_routes::refs::RouteHelper],
) -> Vec<String> {
    helpers
        .iter()
        .find(|helper| helper.name == callee)
        .map(|helper| helper.patterns.clone())
        .unwrap_or_default()
}

fn imported_route_helper_patterns(
    path: &Path,
    callee: &str,
    imports: &[crate::codebase::ts_routes::refs::RouteHelperImport],
    facts: &dyn TsFactLookup,
    resolver: &crate::codebase::ts_resolver::ImportResolver<'_>,
) -> Vec<String> {
    if let Some((namespace, member)) = callee.split_once('.') {
        return imports
            .iter()
            .filter(|import| import.local == namespace)
            .find_map(|import| {
                if import.imported == "*" {
                    route_helper_patterns_from_import(path, member, import, facts, resolver, 0)
                } else {
                    route_helper_namespace_member_patterns(
                        path,
                        &import.imported,
                        member,
                        import,
                        facts,
                        resolver,
                        0,
                    )
                }
            })
            .unwrap_or_default();
    }

    imports
        .iter()
        .find(|import| import.local == callee && import.imported != "*")
        .and_then(|import| {
            route_helper_patterns_from_import(path, &import.imported, import, facts, resolver, 0)
        })
        .unwrap_or_default()
}

fn route_helper_patterns_from_import(
    path: &Path,
    helper_name: &str,
    import: &crate::codebase::ts_routes::refs::RouteHelperImport,
    facts: &dyn TsFactLookup,
    resolver: &crate::codebase::ts_resolver::ImportResolver<'_>,
    depth: usize,
) -> Option<Vec<String>> {
    if depth > 4 {
        return None;
    }
    let target = resolver.resolve(&import.source, path)?;
    let target_facts = facts.get_ts_facts(&target)?;
    if let Some(patterns) = target_facts
        .route_helpers
        .iter()
        .find(|helper| helper.name == helper_name)
        .map(|helper| helper.patterns.clone())
    {
        return Some(patterns);
    }
    for reexport in target_facts
        .route_helper_imports
        .iter()
        .filter(|candidate| candidate.local == helper_name && candidate.imported != "*")
    {
        if let Some(patterns) = route_helper_patterns_from_import(
            &target,
            &reexport.imported,
            reexport,
            facts,
            resolver,
            depth + 1,
        ) {
            return Some(patterns);
        }
    }
    for reexport in target_facts
        .route_helper_imports
        .iter()
        .filter(|candidate| candidate.local == "*" && candidate.imported == "*")
    {
        if let Some(patterns) =
            route_helper_patterns_from_import(&target, helper_name, reexport, facts, resolver, depth + 1)
        {
            return Some(patterns);
        }
    }
    None
}

fn route_helper_namespace_member_patterns(
    path: &Path,
    namespace: &str,
    member: &str,
    import: &crate::codebase::ts_routes::refs::RouteHelperImport,
    facts: &dyn TsFactLookup,
    resolver: &crate::codebase::ts_resolver::ImportResolver<'_>,
    depth: usize,
) -> Option<Vec<String>> {
    if depth > 4 {
        return None;
    }
    let target = resolver.resolve(&import.source, path)?;
    let target_facts = facts.get_ts_facts(&target)?;
    for reexport in target_facts
        .route_helper_imports
        .iter()
        .filter(|candidate| candidate.local == namespace && candidate.imported == "*")
    {
        if let Some(patterns) =
            route_helper_patterns_from_import(&target, member, reexport, facts, resolver, depth + 1)
        {
            return Some(patterns);
        }
    }
    None
}
