use crate::playwright::analysis::cli_run::run;
use crate::playwright::analysis::output::build_related_report;
use crate::playwright::analysis::pipeline::analyze_with_policy;
use crate::playwright::analysis::types::{Analysis, UniqueSelectorPolicy};
use crate::playwright::cli::{Command, PlaywrightArgs as Cli};
use crate::playwright::config::Settings;
use crate::playwright::playwright_tests;
use crate::playwright::test_support::fixture_path;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn analyze(root: &Path, settings: &Settings) -> Result<Analysis> {
    analyze_with_policy(
        root,
        settings,
        playwright_tests::TestPolicy::default(),
        UniqueSelectorPolicy::default(),
    )
}

#[test]
fn text_locators_create_approximate_related_and_coverage_edges_with_route_signal() {
    let root = fixture_path(&["nextjs-selectors", "selector-text-locator"]);
    let cli = Cli {
        root: root.clone(),
        config: None,
        playwright_config: vec![],
        project: None,
        json: true,
        assert_conditional_tests: false,
        allow_skipped_tests: false,
        assert_unique_test_ids: false,
        assert_unique_html_ids: false,
        command: Command::Check,
    };
    assert_eq!(run(cli).unwrap(), ExitCode::from(1));

    let settings = Settings {
        frontend_root: "web/app".to_string(),
        playwright_configs: vec![],
        project: None,
        test_include: vec![],
        test_exclude: vec![],
        ignore_routes: vec![],
        rewrites: vec![],
        navigation_helpers: vec![],
        selector_attributes: vec!["data-testid".to_string(), "data-pw".to_string()],
        test_id_attribute_override: None,
        component_selector_attributes: BTreeMap::new(),
        html_ids: false,
        selector_roots: vec!["web/app".to_string()],
        selector_include: vec![],
        selector_exclude: vec![],
    };
    let analysis = analyze(&root, &settings).unwrap();
    assert_eq!(analysis.coverage.summary.covered_selectors, 5);
    assert_eq!(analysis.coverage.summary.uncovered_selectors, 5);
    assert_locator_text_edge(
        &analysis,
        "web/app/components/discuss-button.tsx",
        "Discuss",
        "discuss-in-community-button",
    );
    assert!(analysis.edges.edges.iter().any(|edge| {
        matches!(
            edge,
            crate::playwright::analysis::types::Edge::LocatorText {
                app_file,
                text,
                reasons,
                ..
            } if app_file.as_ref() == "web/app/components/discuss-button.tsx"
                && text == "Discuss"
                && reasons.contains(&"adjacent-selector".to_string())
        )
    }));
    // "Email" should cover only the input-associated selector, not the button
    // selector; "Send" separately covers the submit input selector.
    assert!(!locator_text_edge_exists(
        &analysis,
        "web/app/components/discuss-button.tsx",
        "Email",
        "email-button"
    ));
    assert_locator_text_edge(
        &analysis,
        "web/app/components/discuss-button.tsx",
        "Email",
        "email-input",
    );
    assert_locator_text_edge(
        &analysis,
        "web/app/components/discuss-button.tsx",
        "Send",
        "submit-input",
    );
    assert!(!locator_text_edge_exists(
        &analysis,
        "web/app/components/discuss-button.tsx",
        "Hidden action",
        "hidden-role-button"
    ));
    assert_locator_text_edge(
        &analysis,
        "web/app/components/discuss-button.tsx",
        "Aria hidden action",
        "aria-hidden-role-button",
    );

    let mut html_id_settings = settings.clone();
    html_id_settings.html_ids = true;
    let html_id_analysis = analyze(&root, &html_id_settings).unwrap();
    assert_eq!(html_id_analysis.coverage.summary.covered_selectors, 7);
    assert_eq!(html_id_analysis.coverage.summary.uncovered_selectors, 5);
    assert_locator_text_edge(
        &html_id_analysis,
        "web/app/components/discuss-button.tsx",
        "save",
        "save-button",
    );

    let related = build_related_report(
        &root,
        &analysis.edges.edges,
        &[PathBuf::from("web/app/components/discuss-button.tsx")],
    );
    assert_eq!(related.tests, vec!["tests/e2e/app.spec.ts"]);

    let unrelated = build_related_report(
        &root,
        &analysis.edges.edges,
        &[PathBuf::from("web/app/components/unreachable-discuss.tsx")],
    );
    assert!(unrelated.tests.is_empty());
}

fn assert_locator_text_edge(analysis: &Analysis, app_file: &str, text: &str, selector_value: &str) {
    assert!(locator_text_edge_exists(
        analysis,
        app_file,
        text,
        selector_value
    ));
}

fn locator_text_edge_exists(
    analysis: &Analysis,
    app_file: &str,
    text: &str,
    selector_value: &str,
) -> bool {
    analysis.edges.edges.iter().any(|edge| {
        matches!(
            edge,
            crate::playwright::analysis::types::Edge::LocatorText {
                app_file: edge_app_file,
                text: edge_text,
                selector_refs,
                reasons,
                ..
            } if edge_app_file.as_ref() == app_file
                && edge_text == text
                && selector_refs.iter().any(|selector| selector.value == selector_value)
                && reasons.contains(&"route-signal".to_string())
        )
    })
}
