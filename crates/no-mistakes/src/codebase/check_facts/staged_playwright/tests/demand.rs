use super::*;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_source::facts::TsFactPlan;

const SCOPED: &[&str] = &["src/scoped.ts", "src/scoped.spec.ts"];
const GRAPH: &[&str] = &[
    "src/scoped.ts",
    "src/scoped.spec.ts",
    "graph/entry.ts",
    "graph/leaf.ts",
    "graph/visible.tsx",
];

#[test]
fn direct_selectors_do_not_upgrade_for_unrelated_visible_text() {
    let mut playwright = PlaywrightFactPlan::default();
    add_test(&mut playwright, "src/scoped.spec.ts", TestPolicy::default());

    let facts = collect_facts(SCOPED, GRAPH, CheckFactPlan::default(), playwright);

    assert!(facts.graph_plan().is_empty());
    assert_eq!(facts.stats.files_parsed, 1);
    assert!(facts.ts.contains_key(&root().join("src/scoped.spec.ts")));
    assert!(!facts.ts.contains_key(&root().join("graph/visible.tsx")));
    assert!(!facts
        .ts
        .get(&root().join("src/scoped.spec.ts"))
        .expect("scoped test facts")
        .ts
        .imports
        .is_empty());
}

#[test]
fn eligible_text_locator_upgrades_every_graph_file_once() {
    let graph = [
        "src/scoped.ts",
        "graph/entry.ts",
        "graph/leaf.ts",
        "graph/graph.spec.ts",
    ];
    let mut playwright = PlaywrightFactPlan::default();
    add_test(
        &mut playwright,
        "graph/graph.spec.ts",
        TestPolicy::default(),
    );

    let facts = collect_facts(
        &["src/scoped.ts"],
        &graph,
        CheckFactPlan::default(),
        playwright,
    );

    assert!(facts.graph_plan().covers(TsFactPlan {
        imports: true,
        ..TsFactPlan::default()
    }));
    assert_eq!(facts.stats.files_parsed, graph.len());
    assert_eq!(facts.stats.files_discovered, graph.len());
    for path in paths(&graph) {
        assert!(
            facts.get_ts_facts(&path).is_some(),
            "missing {}",
            path.display()
        );
    }
}

#[test]
fn skipped_and_teardown_only_locators_do_not_upgrade_imports() {
    let mut playwright = PlaywrightFactPlan::default();
    add_test(
        &mut playwright,
        "tests/skipped.spec.ts",
        TestPolicy::default(),
    );

    let facts = collect_facts(
        &["src/scoped.ts"],
        &["src/scoped.ts", "graph/entry.ts"],
        CheckFactPlan::default(),
        playwright,
    );

    assert!(facts.graph_plan().is_empty());
    assert_eq!(facts.stats.files_parsed, 1);
    assert!(!facts.ts.contains_key(&root().join("graph/entry.ts")));
}

#[test]
fn route_only_variant_caches_urls_without_upgrading_text_imports() {
    let mut playwright = PlaywrightFactPlan::default();
    playwright.add_file(PlaywrightFactSelection {
        path: root().join("tests/route-text.spec.ts"),
        navigation_helpers: &[],
        selector_attributes: &["data-testid".to_string()],
        component_selector_attributes: &BTreeMap::new(),
        html_ids: false,
        test_id_attributes: &["data-testid".to_string()],
        policy: TestPolicy::default(),
        demands_text_imports: false,
    });

    let facts = collect_facts(
        &[],
        &["tests/route-text.spec.ts", "graph/entry.ts"],
        CheckFactPlan::default(),
        playwright,
    );
    let cached = facts.ts[&root().join("tests/route-text.spec.ts")]
        .playwright
        .as_ref()
        .unwrap()
        .all()
        .pop()
        .unwrap();

    assert!(facts.graph_plan().is_empty());
    assert_eq!(facts.stats.files_parsed, 1);
    assert_eq!(cached.urls()[0].value, "/route-only");
    assert!(!cached.text_locators().is_empty());
    assert!(!facts.ts.contains_key(&root().join("graph/entry.ts")));
}
