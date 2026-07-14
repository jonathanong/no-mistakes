use super::selection::rule_selections;
use crate::codebase::check_facts::PlaywrightFactPlan;
use crate::config::v2::NoMistakesConfig;
use crate::playwright::analysis::discover::discover_test_files_from_visible;
use crate::playwright::config;
use crate::playwright::fsutil::VisiblePathSnapshot;
use crate::playwright::playwright_config;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Request-scoped Playwright preparation shared by `check` fact collection
/// and rule execution. The snapshot is intentionally in-memory and is dropped
/// after the invocation.
pub struct PreparedPlaywrightRules {
    pub(super) snapshot: Arc<VisiblePathSnapshot>,
    pub(super) selections: Vec<PreparedRuleSelection>,
    fact_plan: PlaywrightFactPlan,
}

pub(super) struct PreparedRuleSelection {
    pub(super) selection: super::selection::RuleSelection,
    pub(super) settings: config::Settings,
}

impl PreparedPlaywrightRules {
    pub fn fact_plan(&self) -> PlaywrightFactPlan {
        self.fact_plan.clone()
    }

    pub(crate) fn report_view(
        &self,
        project: Option<&str>,
        scan_html_ids: bool,
    ) -> Option<(config::Settings, PlaywrightFactPlan)> {
        let settings = self
            .selections
            .iter()
            .find(|selection| selection.settings.project.as_deref() == project)?
            .settings
            .clone();
        let mut fact_plan = self.fact_plan.clone();
        if scan_html_ids {
            fact_plan.require_html_id_scan(&settings);
        }
        Some((settings, fact_plan))
    }
}

pub fn prepare(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
) -> Result<Option<PreparedPlaywrightRules>> {
    let snapshot = Arc::new(VisiblePathSnapshot::new(root));
    let paths = snapshot.paths_for(root);
    let sources = snapshot.source_store_for(root);
    let tsconfig = crate::codebase::ts_resolver::resolve_tsconfig_from_visible_and_sources(
        None, root, &paths, &sources,
    )?;
    let workspace = crate::codebase::workspaces::load_indexed_from_source_store(root, &sources)
        .unwrap_or_default();
    prepare_with_settings(
        root,
        config,
        snapshot,
        Arc::new(tsconfig),
        Arc::new(workspace),
        |project, snapshot| {
            config::load_settings_from_visible(root, config_path, &[], project, snapshot)
        },
    )
}

/// Prepare Playwright rule facts from the invocation's canonical candidates.
#[doc(hidden)]
pub fn prepare_from_snapshot(
    root: &Path,
    _config_path: Option<&Path>,
    config: &NoMistakesConfig,
    snapshot: Arc<VisiblePathSnapshot>,
    tsconfig: Arc<crate::codebase::ts_resolver::TsConfig>,
) -> Result<Option<PreparedPlaywrightRules>> {
    let workspace = Arc::new(
        crate::codebase::workspaces::load_indexed_from_source_store(
            root,
            &snapshot.source_store_for(root),
        )
        .unwrap_or_default(),
    );
    prepare_with_settings(
        root,
        config,
        snapshot,
        tsconfig,
        workspace,
        |project, snapshot| config::settings_from_loaded_v2(root, config, &[], project, snapshot),
    )
}

fn prepare_with_settings(
    root: &Path,
    config: &NoMistakesConfig,
    snapshot: Arc<VisiblePathSnapshot>,
    tsconfig: Arc<crate::codebase::ts_resolver::TsConfig>,
    workspace: Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
    mut settings_for_project: impl FnMut(
        Option<String>,
        &VisiblePathSnapshot,
    ) -> Result<config::Settings>,
) -> Result<Option<PreparedPlaywrightRules>> {
    let selections = rule_selections(config);
    if selections.is_empty() {
        return Ok(None);
    }
    let prepared_settings = selections
        .into_iter()
        .map(|selection| {
            let settings =
                settings_for_project(selection.playwright_project.clone(), snapshot.as_ref())?;
            Ok((selection, settings))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut config_paths = prepared_settings
        .iter()
        .flat_map(|(_, settings)| settings.playwright_configs.iter().cloned())
        .collect::<Vec<_>>();
    config_paths.sort();
    config_paths.dedup();
    let loaded_configs = playwright_config::load_configs(root, &config_paths)?;

    let mut fact_plan = PlaywrightFactPlan::default();
    let mut test_files_by_project = BTreeMap::new();
    let mut prepared_selections = Vec::new();
    for (selection, settings) in prepared_settings {
        let test_files = add_settings_facts(
            root,
            &settings,
            &loaded_configs,
            snapshot.as_ref(),
            &mut fact_plan,
            selection.unique_html_ids,
        )?;
        test_files_by_project.insert(settings.project.clone(), test_files);
        prepared_selections.push(PreparedRuleSelection {
            selection,
            settings,
        });
    }
    fact_plan.set_test_files_by_project(test_files_by_project.into_iter().collect());
    fact_plan.configure_module_resolution(tsconfig, workspace, snapshot.as_ref(), root);
    Ok(Some(PreparedPlaywrightRules {
        snapshot,
        selections: prepared_selections,
        fact_plan,
    }))
}

fn add_settings_facts(
    root: &Path,
    settings: &config::Settings,
    loaded_configs: &[(PathBuf, playwright_config::PlaywrightConfig)],
    snapshot: &VisiblePathSnapshot,
    fact_plan: &mut PlaywrightFactPlan,
    scan_html_ids: bool,
) -> Result<Arc<Vec<crate::playwright::analysis::context::DiscoveredTestFile>>> {
    let playwright = playwright_config::select_loaded(
        root,
        &settings.playwright_configs,
        settings.project.as_deref(),
        loaded_configs,
    )?;
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
    Ok(Arc::new(test_files))
}

pub fn fact_plan(
    root: &Path,
    config_path: Option<&Path>,
    config: &NoMistakesConfig,
) -> Result<Option<PlaywrightFactPlan>> {
    Ok(prepare(root, config_path, config)?.map(|prepared| prepared.fact_plan()))
}
