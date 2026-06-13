use crate::playwright::analysis::coverage::build_coverage as build_coverage_report;
use crate::playwright::analysis::types::{
    CoverageInputs, FetchIndex, SelectorHelperReference, SelectorHelperReferenceWithValue,
    UniqueSelectorPolicy,
};
use crate::playwright::config::Settings;
use crate::playwright::selectors::{self, AppSelectorValue};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn default_settings(selector_attributes: Vec<String>) -> Settings {
    Settings {
        frontend_root: "web/app".to_string(),
        playwright_configs: vec![],
        project: None,
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![],
        navigation_helpers: vec![],
        selector_attributes,
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec!["web/app".to_string()],
        selector_include: vec![],
        selector_exclude: vec![],
    }
}

#[test]
fn helper_reference_hints_are_scoped_to_test_id_attributes() {
    let root = Path::new("/repo");
    let app_selectors = vec![
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/page.tsx"),
            attribute: "data-testid".to_string(),
            value: AppSelectorValue::Exact("save".to_string()),
        },
        selectors::AppSelector {
            file: PathBuf::from("/repo/web/app/page.tsx"),
            attribute: "id".to_string(),
            value: AppSelectorValue::Exact("save".to_string()),
        },
    ];
    let settings = default_settings(vec!["data-testid".to_string(), "id".to_string()]);
    let helper_references = vec![SelectorHelperReferenceWithValue {
        attribute: "data-testid".to_string(),
        value: "save".to_string(),
        reference: SelectorHelperReference {
            test_file: std::sync::Arc::new("tests/e2e/app.spec.ts".to_string()),
            line: 4,
            call: "getSaveLocator(...)".to_string(),
        },
    }];

    let report = build_coverage_report(CoverageInputs {
        root,
        routes: &[],
        app_selectors: &app_selectors,
        app_selector_occurrences: &app_selectors,
        edges: &[],
        helper_references: &helper_references,
        settings: &settings,
        unique_selector_policy: UniqueSelectorPolicy::default(),
        fetch_index: &FetchIndex::new(),
    });

    let test_id_selector = report
        .selectors
        .iter()
        .find(|selector| selector.attribute == "data-testid")
        .expect("data-testid selector");
    let id_selector = report
        .selectors
        .iter()
        .find(|selector| selector.attribute == "id")
        .expect("id selector");

    assert_eq!(test_id_selector.helper_references.len(), 1);
    assert!(id_selector.helper_references.is_empty());
}
