use super::rule_selections;
use crate::codebase::check_facts::{PlaywrightFactPlan, PlaywrightFactSelection};
use crate::config::v2::NoMistakesConfig;
use crate::playwright::analysis::discover::discover_test_files_from_visible;
use crate::playwright::{config, playwright_config, playwright_tests};
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Copy, Default)]
pub struct PlaywrightFactConsumers {
    pub graph_selectors: bool,
    pub graph_routes: bool,
}

#[doc(hidden)]
pub fn fact_plan_for_consumers(
    root: &Path,
    _config_path: Option<&Path>,
    config: &NoMistakesConfig,
    consumers: PlaywrightFactConsumers,
) -> Result<Option<PlaywrightFactPlan>> {
    let selections = rule_selections(config);
    if selections.is_empty() && !consumers.graph_selectors && !consumers.graph_routes {
        return Ok(None);
    }

    let snapshot = crate::playwright::fsutil::VisiblePathSnapshot::new(root);
    let mut prepared = selections
        .into_iter()
        .map(|selection| {
            let settings = config::settings_from_loaded_v2(
                root,
                config,
                &[],
                selection.playwright_project,
                &snapshot,
            )?;
            Ok((settings, true, selection.unique_html_ids))
        })
        .collect::<Result<Vec<_>>>()?;
    if consumers.graph_selectors || consumers.graph_routes {
        prepared.push((
            config::settings_from_loaded_v2(root, config, &[], None, &snapshot)?,
            consumers.graph_selectors,
            false,
        ));
    }

    let mut config_paths = prepared
        .iter()
        .flat_map(|(settings, _, _)| settings.playwright_configs.iter().cloned())
        .collect::<Vec<_>>();
    config_paths.sort();
    config_paths.dedup();
    let loaded_configs = playwright_config::load_configs(root, &config_paths)?;

    let mut plan = PlaywrightFactPlan::default();
    let mut test_files_by_project = BTreeMap::new();
    for (settings, demands_text_imports, scan_html_ids) in prepared {
        let playwright = playwright_config::select_loaded(
            root,
            &settings.playwright_configs,
            settings.project.as_deref(),
            &loaded_configs,
        )?;
        let test_files = discover_test_files_from_visible(root, &settings, &playwright, &snapshot)?;
        for test_file in &test_files {
            let attributes = test_file.test_id_attributes();
            plan.add_file(PlaywrightFactSelection {
                path: test_file.path.clone(),
                navigation_helpers: &settings.navigation_helpers,
                selector_attributes: &settings.selector_attributes,
                component_selector_attributes: &settings.component_selector_attributes,
                html_ids: settings.html_ids,
                test_id_attributes: &attributes,
                policy: playwright_tests::TestPolicy {
                    assert_conditional_tests: false,
                    allow_skipped_tests: false,
                },
                demands_text_imports,
            });
        }
        plan.add_source_settings(root, settings.clone(), scan_html_ids, &snapshot)?;
        test_files_by_project.insert(settings.project.clone(), Arc::new(test_files));
    }
    plan.set_test_files_by_project(test_files_by_project.into_iter().collect());
    Ok(Some(plan))
}
