use crate::codebase::check_facts::PlaywrightFactPlan;
use crate::playwright::analysis::discover::discover_test_files_from_visible;
use crate::playwright::fsutil::VisiblePathSnapshot;
use crate::playwright::{config, playwright_config};
use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) fn add_settings_facts(
    root: &Path,
    settings: &config::Settings,
    loaded_configs: &[(PathBuf, playwright_config::PlaywrightConfig)],
    snapshot: &VisiblePathSnapshot,
    fact_plan: &mut PlaywrightFactPlan,
    scan_html_ids: bool,
) -> Result<(
    Arc<Vec<crate::playwright::analysis::context::DiscoveredTestFile>>,
    Vec<String>,
)> {
    let playwright = playwright_config::select_loaded(
        root,
        &settings.playwright_configs,
        settings.project.as_deref(),
        loaded_configs,
    );
    let playwright = playwright?;
    let test_files = discover_test_files_from_visible(root, settings, &playwright, snapshot)?;
    for test_file in &test_files {
        let attributes = test_file.test_id_attributes();
        fact_plan.add_file(crate::codebase::check_facts::PlaywrightFactSelection {
            path: test_file.path.clone(),
            navigation_helpers: &settings.navigation_helpers,
            selector_wrappers: &settings.selector_wrappers,
            selector_attributes: &settings.selector_attributes,
            component_selector_attributes: &settings.component_selector_attributes,
            html_ids: settings.html_ids,
            test_id_attributes: &attributes,
            policy: crate::playwright::playwright_tests::TestPolicy {
                assert_conditional_tests: false,
                allow_skipped_tests: false,
            },
            demands_text_imports: true,
        });
    }
    fact_plan.add_source_settings(root, settings.clone(), scan_html_ids, snapshot)?;
    let warnings = playwright.matcher_warnings(root);
    fact_plan.add_matcher_warnings(settings.project.clone(), warnings.clone());
    Ok((Arc::new(test_files), warnings))
}
