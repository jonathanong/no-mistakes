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
    resolver: &dyn crate::codebase::ts_resolver::ImportResolution,
    graph_files: &GraphFiles,
) -> Vec<String> {
    let mut patterns: Vec<_> = route_helper_ref_patterns_with_lines(
        path,
        file_facts,
        facts,
        resolver,
        graph_files,
    )
        .into_iter()
        .map(|(_, pattern)| pattern)
        .collect();
    patterns.sort();
    patterns.dedup();
    patterns
}

pub(crate) fn route_helper_ref_patterns_with_lines(
    path: &Path,
    file_facts: &crate::codebase::ts_source::facts::TsFileFacts,
    facts: &dyn TsFactLookup,
    resolver: &dyn crate::codebase::ts_resolver::ImportResolution,
    graph_files: &GraphFiles,
) -> Vec<(u32, String)> {
    let inputs = RouteHelperResolutionInputs {
        facts,
        resolver,
        graph_files,
    };
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
            inputs,
        ));
        patterns.extend(
            route_helper_ref_wrapped_patterns(helper_ref, helper_patterns)
                .into_iter()
                .map(|pattern| (helper_ref.line, pattern)),
        );
    }
    patterns.sort();
    patterns.dedup();
    patterns
}

#[derive(Clone, Copy)]
struct RouteHelperResolutionInputs<'a> {
    facts: &'a dyn TsFactLookup,
    resolver: &'a dyn crate::codebase::ts_resolver::ImportResolution,
    graph_files: &'a GraphFiles,
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

include!("edge_route_helper_refs_imports.rs");
