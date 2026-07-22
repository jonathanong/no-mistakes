fn imported_route_helper_patterns(path: &Path, callee: &str, imports: &[crate::codebase::ts_routes::refs::RouteHelperImport], inputs: RouteHelperResolutionInputs<'_>) -> Vec<String> {
    if let Some((namespace, member)) = callee.split_once('.') {
        return imports.iter().filter(|import| import.local == namespace).find_map(|import| {
            if import.imported == "*" { route_helper_patterns_from_import(path, member, import, inputs, 0) }
            else { route_helper_namespace_member_patterns(path, &import.imported, member, import, inputs, 0) }
        }).unwrap_or_default();
    }
    imports.iter().find(|import| import.local == callee && import.imported != "*")
        .and_then(|import| route_helper_patterns_from_import(path, &import.imported, import, inputs, 0)).unwrap_or_default()
}

fn route_helper_patterns_from_import(path: &Path, helper_name: &str, import: &crate::codebase::ts_routes::refs::RouteHelperImport, inputs: RouteHelperResolutionInputs<'_>, depth: usize) -> Option<Vec<String>> {
    if depth > 4 { return None; }
    let target = inputs.graph_files.visible_path(&inputs.resolver.resolve(&import.source, path)?)?;
    let target_facts = inputs.facts.get_ts_facts(target)?;
    if let Some(patterns) = target_facts.route_helpers.iter().find(|helper| helper.name == helper_name).map(|helper| helper.patterns.clone()) { return Some(patterns); }
    for reexport in target_facts.route_helper_imports.iter().filter(|candidate| candidate.local == helper_name && candidate.imported != "*") {
        if let Some(patterns) = route_helper_patterns_from_import(target, &reexport.imported, reexport, inputs, depth + 1) { return Some(patterns); }
    }
    for reexport in target_facts.route_helper_imports.iter().filter(|candidate| candidate.local == "*" && candidate.imported == "*") {
        if let Some(patterns) = route_helper_patterns_from_import(target, helper_name, reexport, inputs, depth + 1) { return Some(patterns); }
    }
    None
}

fn route_helper_namespace_member_patterns(path: &Path, namespace: &str, member: &str, import: &crate::codebase::ts_routes::refs::RouteHelperImport, inputs: RouteHelperResolutionInputs<'_>, depth: usize) -> Option<Vec<String>> {
    if depth > 4 { return None; }
    let target = inputs.graph_files.visible_path(&inputs.resolver.resolve(&import.source, path)?)?;
    let target_facts = inputs.facts.get_ts_facts(target)?;
    for reexport in target_facts.route_helper_imports.iter().filter(|candidate| candidate.local == namespace && candidate.imported == "*") {
        if let Some(patterns) = route_helper_patterns_from_import(target, member, reexport, inputs, depth + 1) { return Some(patterns); }
    }
    None
}
