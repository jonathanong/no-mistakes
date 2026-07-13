use crate::playwright::analysis::pipeline::analyze_with_policy;
use crate::playwright::analysis::pipeline_selectors::analyze_selectors_with_policy;
use crate::playwright::analysis::types::UniqueSelectorPolicy;
use crate::playwright::playwright_tests::TestPolicy;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/playwright/error-order")
            .join(name)
            .join("fixture"),
    )
}

fn analyze_fixture(
    name: &str,
    require_routes: bool,
) -> anyhow::Result<crate::playwright::analysis::types::Analysis> {
    let root = fixture(name);
    let settings = crate::playwright::config::load_settings(&root, None, &[], None)?;
    let policy = TestPolicy::default();
    let unique = UniqueSelectorPolicy::default();
    if require_routes {
        analyze_with_policy(&root, &settings, policy, unique)
    } else {
        analyze_selectors_with_policy(&root, &settings, policy, unique)
    }
}

#[test]
fn playwright_config_error_precedes_malformed_app_source() {
    for require_routes in [true, false] {
        let error = match analyze_fixture("config-before-app", require_routes) {
            Ok(_) => panic!("fixture must fail"),
            Err(error) => error,
        };
        assert!(
            format!("{error:#}").contains("missing.playwright.config.ts"),
            "expected Playwright config error first, got: {error:#}"
        );
    }
}

#[test]
fn malformed_app_source_precedes_malformed_test_source() {
    for require_routes in [true, false] {
        let error = match analyze_fixture("app-before-test", require_routes) {
            Ok(_) => panic!("fixture must fail"),
            Err(error) => error,
        };
        let message = format!("{error:#}");
        assert!(
            message.contains("bad-button.tsx"),
            "expected app selector parse error first, got: {message}"
        );
        assert!(!message.contains("app.spec.ts"));
    }
}

#[test]
fn malformed_route_source_propagates_from_fetch_collection() {
    let root = fixture("malformed-route-fetch");
    let mut settings = crate::playwright::config::load_settings(&root, None, &[], None).unwrap();
    settings.selector_attributes.clear();
    settings.component_selector_attributes.clear();
    settings.html_ids = false;

    let error = match analyze_with_policy(
        &root,
        &settings,
        TestPolicy::default(),
        UniqueSelectorPolicy::default(),
    ) {
        Ok(_) => panic!("malformed route source must fail fetch collection"),
        Err(error) => error,
    };
    assert!(format!("{error:#}").contains("page.tsx"));
}
