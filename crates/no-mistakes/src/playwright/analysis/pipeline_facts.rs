use super::types::UniqueSelectorPolicy;
use crate::playwright::config;
use crate::playwright::fsutil::VisiblePathSnapshot;
use crate::playwright::playwright_tests;
use anyhow::Result;
use std::path::Path;

pub(crate) fn standalone_facts(
    root: &Path,
    settings: &config::Settings,
    unique_selector_policy: UniqueSelectorPolicy,
    snapshot: &VisiblePathSnapshot,
) -> Result<crate::codebase::check_facts::CheckFactMap> {
    let mut fact_plan = standalone_fact_plan(root, settings, unique_selector_policy, snapshot)?;
    if !settings.selector_wrappers.is_empty() {
        let paths = snapshot.paths_for(root);
        let sources = snapshot.source_store_for(root);
        let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
            None, root, &paths, &sources,
        )?;
        let workspace = crate::codebase::workspaces::load_indexed_from_source_store(root, &sources)
            .unwrap_or_default();
        fact_plan.configure_module_resolution(
            std::sync::Arc::new(tsconfig),
            std::sync::Arc::new(workspace),
            snapshot,
            root,
        );
    }
    let mut files = snapshot
        .paths_for(root)
        .iter()
        .filter(|path| crate::codebase::dependencies::extract::is_indexable(path))
        .cloned()
        .collect::<Vec<_>>();
    files.extend(fact_plan.paths().cloned());
    files.sort();
    files.dedup();
    Ok(
        crate::codebase::check_facts::collect_check_facts_with_playwright(
            root,
            files,
            crate::codebase::check_facts::CheckFactPlan::default(),
            Some(fact_plan),
        ),
    )
}

pub(crate) fn standalone_fact_plan(
    root: &Path,
    settings: &config::Settings,
    unique_selector_policy: UniqueSelectorPolicy,
    snapshot: &VisiblePathSnapshot,
) -> Result<crate::codebase::check_facts::PlaywrightFactPlan> {
    let playwright = crate::playwright::playwright_config::load_many(
        root,
        &settings.playwright_configs,
        settings.project.as_deref(),
    )?;
    let test_files = crate::playwright::analysis::discover::discover_test_files_from_visible(
        root,
        settings,
        &playwright,
        snapshot,
    )?;
    let mut fact_plan = crate::codebase::check_facts::PlaywrightFactPlan::default();
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
            policy: playwright_tests::TestPolicy {
                assert_conditional_tests: false,
                allow_skipped_tests: false,
            },
            demands_text_imports: true,
        });
    }
    fact_plan.set_test_files_by_project(vec![(
        settings.project.clone(),
        std::sync::Arc::new(test_files),
    )]);
    fact_plan.add_source_settings(
        root,
        settings.clone(),
        unique_selector_policy.html_ids && !settings.html_ids,
        snapshot,
    )?;
    Ok(fact_plan)
}
