use crate::codebase::check_facts::{
    collect_check_facts_with_graph_files_and_playwright, CheckFactMap, CheckFactPlan,
    PlaywrightFactPlan, PlaywrightFactSelection,
};
use crate::playwright::playwright_tests::TestPolicy;
use std::collections::BTreeMap;
use std::path::PathBuf;

mod demand;
mod graph;
mod reuse;
mod settings;

fn root() -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/codebase/staged-playwright/fixture"),
    )
}

fn paths(names: &[&str]) -> Vec<PathBuf> {
    let root = root();
    names.iter().map(|name| root.join(name)).collect()
}

fn occurrence_settings(
    navigation_helpers: &[&str],
    selector_attributes: &[&str],
) -> crate::playwright::config::Settings {
    let mut settings =
        crate::playwright::config::test_support::load_settings(&root(), None, &[], None).unwrap();
    settings.navigation_helpers = navigation_helpers
        .iter()
        .map(|value| value.to_string())
        .collect();
    settings.selector_attributes = selector_attributes
        .iter()
        .map(|value| value.to_string())
        .collect();
    settings.component_selector_attributes.clear();
    settings.html_ids = false;
    settings
}

fn add_test(plan: &mut PlaywrightFactPlan, name: &str, policy: TestPolicy) {
    plan.add_file(PlaywrightFactSelection {
        path: root().join(name),
        navigation_helpers: &[],
        selector_attributes: &["data-testid".to_string()],
        component_selector_attributes: &BTreeMap::new(),
        html_ids: false,
        test_id_attributes: &["data-testid".to_string()],
        policy,
        demands_text_imports: true,
    });
}

fn collect_facts(
    scoped: &[&str],
    graph: &[&str],
    plan: CheckFactPlan,
    playwright: PlaywrightFactPlan,
) -> CheckFactMap {
    collect_check_facts_with_graph_files_and_playwright(
        &root(),
        paths(scoped),
        paths(graph),
        plan,
        Some(playwright),
    )
}
