use crate::config::v2::schema::RewriteRule;
use crate::playwright::analysis::pipeline::analyze_with_policy_and_optional_facts;
use crate::playwright::analysis::pipeline_options::AnalysisOptions;
use crate::playwright::analysis::types::UniqueSelectorPolicy;
use crate::playwright::config::Settings;
use crate::playwright::playwright_tests::TestPolicy;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn fixture() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/nextjs-rewrites/basic/fixture"),
    )
}

#[test]
fn pipeline_expands_rewrites_into_route_edges() {
    let root = fixture();
    let settings = Settings {
        frontend_root: "app".to_string(),
        playwright_configs: vec![root.join("playwright.config.ts")],
        project: None,
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![
            RewriteRule {
                source: "/posts/:slug*".to_string(),
                destination: "/content/posts/:slug*".to_string(),
            },
            RewriteRule {
                source: "/reviews/:slug*".to_string(),
                destination: "/content/reviews/:slug*".to_string(),
            },
        ],
        navigation_helpers: vec![],
        selector_attributes: vec![],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec![],
        selector_include: vec![],
        selector_exclude: vec![],
    };
    let analysis = analyze_with_policy_and_optional_facts(
        &root,
        &settings,
        TestPolicy::default(),
        UniqueSelectorPolicy::default(),
        AnalysisOptions {
            require_routes: false,
            skip_test_file_errors: false,
            facts: None,
            route_import_candidate: None,
            graph_file_universe: None,
            occurrence_selection:
                crate::playwright::analysis::pipeline_occurrences::CachedOccurrenceSelection::Exact,
        },
    )
    .unwrap();
    let route_patterns: Vec<&str> = analysis
        .coverage
        .routes
        .iter()
        .map(|r| r.route.as_str())
        .collect();
    assert!(route_patterns.contains(&"/posts/**"));
    assert!(route_patterns.contains(&"/reviews/**"));
}
