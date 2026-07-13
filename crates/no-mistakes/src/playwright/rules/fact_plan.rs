use super::rule_selections;
use crate::codebase::check_facts::{PlaywrightFactPlan, PlaywrightFactSelection};
use crate::config::v2::NoMistakesConfig;
use crate::playwright::analysis::discover::discover_test_files;
use crate::playwright::{config, playwright_config, playwright_tests};
use anyhow::Result;
use std::path::Path;

#[derive(Clone, Copy, Default)]
pub struct PlaywrightFactConsumers {
    pub graph_selectors: bool,
    pub graph_routes: bool,
}

pub fn fact_plan(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
) -> Result<Option<PlaywrightFactPlan>> {
    fact_plan_for_consumers(
        root,
        config_path,
        config,
        PlaywrightFactConsumers::default(),
    )
}

#[doc(hidden)]
pub fn fact_plan_for_consumers(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
    consumers: PlaywrightFactConsumers,
) -> Result<Option<PlaywrightFactPlan>> {
    let selections = rule_selections(config);
    if selections.is_empty() && !consumers.graph_selectors && !consumers.graph_routes {
        return Ok(None);
    }
    let mut plan = PlaywrightFactPlan::default();
    for selection in selections {
        add_settings(
            root,
            config_path,
            selection.playwright_project,
            true,
            &mut plan,
        )?;
    }
    if consumers.graph_selectors || consumers.graph_routes {
        let text = consumers.graph_selectors;
        add_settings(root, config_path, None, text, &mut plan)?;
    }
    Ok(Some(plan))
}

fn add_settings(
    root: &Path,
    config_path: Option<&Path>,
    project: Option<String>,
    demands_text_imports: bool,
    plan: &mut PlaywrightFactPlan,
) -> Result<()> {
    let settings = config::load_settings(root, config_path, &[], project)?;
    let playwright = playwright_config::load_many(
        root,
        &settings.playwright_configs,
        settings.project.as_deref(),
    )?;
    for test_file in discover_test_files(root, &settings, &playwright)? {
        let attributes = test_file.test_id_attributes();
        plan.add_file(PlaywrightFactSelection {
            path: test_file.path,
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
    Ok(())
}
