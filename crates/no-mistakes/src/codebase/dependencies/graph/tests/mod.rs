use crate::codebase::ts_source::facts::collect_ts_facts_with_context;

fn package_name_from_spec(spec: &str) -> &str {
    if spec.starts_with('@') {
        let after_scope = spec.trim_start_matches('@');
        let slash_idx = after_scope.find('/').map(|i| i + 1);
        if let Some(idx) = slash_idx {
            let after_first_slash = &after_scope[idx..];
            let end = after_first_slash
                .find('/')
                .map(|i| idx + i + 1)
                .unwrap_or(spec.len());
            &spec[..end]
        } else {
            spec
        }
    } else {
        match spec.find('/') {
            Some(idx) => &spec[..idx],
            None => spec,
        }
    }
}

#[test]
fn playwright_graph_build_has_one_snapshot_construction_site() {
    let builder = [
        include_str!("../builder_core.rs"),
        include_str!("../builder_edges.rs"),
    ]
    .concat();

    assert_eq!(
        builder.matches("VisiblePathSnapshot::from_paths").count(),
        1,
        "route and selector edges must reuse one classified snapshot"
    );
    let shared_cache = include_str!("../../shared_graph_cache.rs");
    assert!(
        shared_cache.contains("visible_paths: Some(input.visible_paths)"),
        "shared traversal must pass the canonical request snapshot into graph construction"
    );
}

include!("build_check_fact_adapters.rs");
include!("core.rs");
include!("scoped_universe.rs");
include!("legacy_symbol_channel.rs");
include!("route_import.rs");
include!("route_import_prepared.rs");
include!("extra_cases.rs");
include!("lazy_import_session.rs");
include!("extra_playwright_routes.rs");
include!("extra_selector.rs");
include!("extra_symbol_scoped.rs");
include!("extra_symbol_defensive.rs");
include!("extra_symbol_helpers.rs");
include!("extra_symbol_visibility.rs");
include!("extra_symbol.rs");
include!("extra_symbol_gitignore.rs");
include!("extra_gitignore_pass3.rs");
include!("types.rs");

mod selector_fact_plan;
mod selector_optimization;
