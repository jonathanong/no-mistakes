use super::*;
use crate::playwright::analysis::pipeline_selectors::test_support::{
    analyze_selectors_with_policy, analyze_selectors_with_policy_and_facts,
    analyze_selectors_with_policy_and_graph, analyze_selectors_with_policy_facts_and_graph,
};
use crate::playwright::analysis::types::{Edge, UniqueSelectorPolicy};
use crate::playwright::rules::{fact_plan_for_consumers, PlaywrightFactConsumers};

#[test]
fn graph_all_projects_and_rule_a_reuse_exact_complete_variants() {
    let root = root();
    let mut config = crate::config::v2::load_v2_config(&root, None).unwrap();
    config
        .rules
        .retain(|rule| rule.tests.playwright.iter().any(|project| project == "a"));
    let playwright = fact_plan_for_consumers(
        &root,
        None,
        &config,
        PlaywrightFactConsumers {
            graph_selectors: true,
            graph_routes: false,
        },
    )
    .unwrap()
    .unwrap();
    let graph_files = paths(&["tests/multi.spec.ts", "graph/b-only.tsx"]);
    let facts = crate::codebase::check_facts::collect_check_facts_with_graph_files_and_playwright(
        &root,
        Vec::new(),
        graph_files.clone(),
        CheckFactPlan::default(),
        Some(playwright),
    );

    assert_rule_a_parity(&root, &config, &facts);
    assert_graph_union_parity(&root, &graph_files, &facts);
}

fn assert_rule_a_parity(
    root: &std::path::Path,
    config: &crate::config::v2::NoMistakesConfig,
    facts: &CheckFactMap,
) {
    let standalone = crate::playwright::rules::check(root, None, config).unwrap();
    let cached = crate::playwright::rules::check_with_facts(root, None, config, facts).unwrap();
    assert_eq!(cached, standalone);

    let settings = crate::playwright::config::test_support::load_settings(
        root,
        None,
        &[],
        Some("a".to_string()),
    )
    .unwrap();
    let standalone = analyze_selectors_with_policy(
        root,
        &settings,
        TestPolicy::default(),
        UniqueSelectorPolicy::default(),
    )
    .unwrap();
    let cached = analyze_selectors_with_policy_and_facts(
        root,
        &settings,
        TestPolicy::default(),
        UniqueSelectorPolicy::default(),
        facts,
    )
    .unwrap();
    assert!(cached.edges.edges == standalone.edges.edges);
    assert!(!cached.edges.edges.iter().any(is_b_only_selector));
}

fn assert_graph_union_parity(
    root: &std::path::Path,
    graph_files: &[std::path::PathBuf],
    facts: &CheckFactMap,
) {
    let settings =
        crate::playwright::config::test_support::load_settings(root, None, &[], None).unwrap();
    let standalone = analyze_selectors_with_policy_and_graph(
        root,
        &settings,
        TestPolicy::default(),
        UniqueSelectorPolicy::default(),
        None,
        graph_files,
    )
    .unwrap();
    let cached = analyze_selectors_with_policy_facts_and_graph(
        root,
        &settings,
        TestPolicy::default(),
        UniqueSelectorPolicy::default(),
        facts,
        None,
        graph_files,
    )
    .unwrap();

    assert!(cached.edges.edges == standalone.edges.edges);
    assert!(
        cached.edges.edges.iter().any(is_b_only_selector),
        "missing B-only edge: {}",
        serde_json::to_string(&cached.edges.edges).unwrap()
    );
}

fn is_b_only_selector(edge: &Edge) -> bool {
    matches!(
        edge,
        Edge::Selector {
            app_file,
            attribute,
            value,
            ..
        } if app_file.as_str() == "graph/b-only.tsx"
            && attribute == "data-b"
            && value == "only-b"
    )
}
