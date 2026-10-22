use super::*;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::ts_source::facts::TsFactPlan;

#[test]
fn finite_only_staged_scopes_parse_only_selected_primary_and_graph_files() {
    let mut selected = paths(&["src/scoped.ts", "graph/entry.ts"]);
    selected.sort();
    let facts = collect_facts(
        &["src/scoped.ts", "src/scoped.spec.ts"],
        &["graph/entry.ts", "graph/leaf.ts"],
        CheckFactPlan {
            call_site_paths: std::sync::Arc::new(selected.clone()),
            ..Default::default()
        },
        PlaywrightFactPlan::default(),
    );
    assert_eq!(facts.stats.files_parsed, 2);
    assert!(facts.graph_plan().is_empty());
    for path in &selected {
        assert!(facts.get_ts_facts(path).is_some());
        assert!(facts.covers_ts_fact_plan_for_file(
            path,
            TsFactPlan {
                call_sites: true,
                ..Default::default()
            }
        ));
    }
    assert!(!facts.ts.contains_key(&root().join("src/scoped.spec.ts")));
    assert!(!facts.ts.contains_key(&root().join("graph/leaf.ts")));
}

#[test]
fn finite_selected_playwright_only_test_retains_its_static_call_literal() {
    let selected = root().join("tests/playwright-only.spec.ts");
    let mut playwright = PlaywrightFactPlan::default();
    add_test(
        &mut playwright,
        "tests/playwright-only.spec.ts",
        TestPolicy::default(),
    );
    let facts = collect_facts(
        &[],
        &[],
        CheckFactPlan {
            call_site_paths: std::sync::Arc::new(vec![selected.clone()]),
            ..Default::default()
        },
        playwright,
    );
    assert_eq!(facts.stats.files_parsed, 1);
    assert!(facts.covers_ts_fact_plan_for_file(
        &selected,
        TsFactPlan {
            call_sites: true,
            ..Default::default()
        }
    ));
    assert!(facts
        .get_ts_facts(&selected)
        .is_some_and(|file| file.call_sites.iter().any(|call| {
            call.callee == "test"
                && call.static_arg_source.as_deref() == Some("\"playwright-only selector\"")
        })));
}

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
        selector_wrappers: &[],
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
