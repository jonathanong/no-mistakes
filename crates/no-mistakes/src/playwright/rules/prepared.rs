use super::selection::rule_selections;
use crate::codebase::check_facts::PlaywrightFactPlan;
use crate::codebase::check_outcome::Prerequisite;
use crate::config::v2::NoMistakesConfig;
use crate::playwright::config;
use crate::playwright::fsutil::VisiblePathSnapshot;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;

type SettingsForProject<'a> = dyn FnMut(Option<String>, Option<String>, &VisiblePathSnapshot) -> Result<config::Settings>
    + 'a;

pub struct PreparedPlaywrightRules {
    pub(super) snapshot: Arc<VisiblePathSnapshot>,
    pub(super) selections: Vec<PreparedRuleSelection>,
    fact_plan: PlaywrightFactPlan,
    warnings: Vec<String>,
    failures: Vec<crate::codebase::check_outcome::CheckFailure>,
}

pub(super) struct PreparedRuleSelection {
    pub(super) selection: super::selection::RuleSelection,
    pub(super) settings: Prerequisite<config::Settings>,
}

impl PreparedPlaywrightRules {
    pub fn failures(&self) -> &[crate::codebase::check_outcome::CheckFailure] {
        &self.failures
    }
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }

    pub fn fact_plan(&self) -> PlaywrightFactPlan {
        self.fact_plan.clone()
    }

    /// An omitted app filter must reuse the prepared project selection.
    pub(crate) fn report_view(
        &self,
        project: Option<&str>,
        app: Option<&str>,
        scan_html_ids: bool,
    ) -> Option<(config::Settings, PlaywrightFactPlan)> {
        let settings = self.selections.iter().find_map(|selection| {
            let Prerequisite::Ready(settings) = &selection.settings else {
                return None;
            };
            (settings.project.as_deref() == project
                && app.is_none_or(|app| selection.selection.app.as_deref() == Some(app)))
            .then(|| settings.clone())
        })?;
        let mut fact_plan = self.fact_plan.clone();
        if scan_html_ids {
            fact_plan.require_html_id_scan(&settings);
        }
        Some((settings, fact_plan))
    }
}

pub(super) fn prepare_with_settings(
    root: &Path,
    config: &NoMistakesConfig,
    snapshot: Arc<VisiblePathSnapshot>,
    tsconfig: Arc<crate::codebase::ts_resolver::TsConfig>,
    workspace: Arc<crate::codebase::workspaces::IndexedWorkspaceMap>,
    options: PreparationOptions,
    settings_for_project: &mut SettingsForProject<'_>,
) -> Result<Option<PreparedPlaywrightRules>> {
    let PreparationOptions {
        tsconfig_catalog,
        partial,
    } = options;
    let root_paths = snapshot.paths_for(root);
    let apps = crate::config::v2::frontend_apps(root, config, &root_paths)?;
    let selections = rule_selections(config, &apps)?;
    if selections.is_empty() {
        return Ok(None);
    }
    let mut failures = Vec::new();
    let mut prepared_settings = Vec::new();
    let mut prepared_selections = Vec::new();
    for selection in selections {
        match settings_for_project(
            selection.playwright_project.clone(),
            selection.app.clone(),
            &snapshot,
        ) {
            Ok(settings) => {
                prepared_settings.push((selection, partial::normalize_settings(root, settings)))
            }
            Err(error) if partial => {
                let failed = partial::selection_failures(&selection, None, &error);
                failures.extend(failed.clone());
                prepared_selections.push(PreparedRuleSelection {
                    selection,
                    settings: Prerequisite::Failed(failed),
                });
            }
            Err(error) => return Err(error),
        }
    }
    let mut config_paths = prepared_settings
        .iter()
        .flat_map(|(_, settings)| settings.playwright_configs.iter().cloned())
        .collect::<Vec<_>>();
    config_paths.sort();
    config_paths.dedup();
    let (loaded_configs, config_failures) =
        partial::load_configs(root, &config_paths, &snapshot, partial)?;
    let mut warnings = Vec::new();
    let mut fact_plan = PlaywrightFactPlan::default();
    fact_plan.skip_dynamic_recovery = super::effort::skip_dynamic_recovery(config, false)?;
    let mut test_files_by_project = BTreeMap::new();
    for (selection, mut settings) in prepared_settings {
        if !config_failures.is_empty()
            && !partial::apply_failures(
                &selection,
                &mut settings,
                &loaded_configs,
                &config_failures,
                &mut failures,
            )
        {
            let failed = failures
                .iter()
                .filter(|failure| partial::selection_consumes(&selection, &failure.consumer))
                .cloned()
                .collect();
            prepared_selections.push(PreparedRuleSelection {
                selection,
                settings: Prerequisite::Failed(failed),
            });
            continue;
        }
        let test_files = add_settings_facts(
            root,
            &settings,
            &loaded_configs,
            snapshot.as_ref(),
            &mut fact_plan,
            selection.unique_html_ids,
        );
        let (test_files, project_warnings) = match test_files {
            Ok(value) => value,
            Err(error) if partial => {
                let failed = partial::selection_failures(&selection, None, &error);
                failures.extend(failed.clone());
                prepared_selections.push(PreparedRuleSelection {
                    selection,
                    settings: Prerequisite::Failed(failed),
                });
                continue;
            }
            Err(error) => return Err(error),
        };
        warnings.extend(project_warnings);
        test_files_by_project.insert(settings.project.clone(), test_files);
        prepared_selections.push(PreparedRuleSelection {
            selection,
            settings: Prerequisite::Ready(settings),
        });
    }
    warnings.sort();
    warnings.dedup();
    fact_plan.set_matcher_warnings(warnings.clone());
    fact_plan.set_test_files_by_project(test_files_by_project.into_iter().collect());
    if let Some(tsconfig_catalog) = tsconfig_catalog {
        fact_plan.configure_module_resolution_with_catalog(
            tsconfig_catalog,
            workspace,
            snapshot.as_ref(),
            root,
        );
    } else {
        fact_plan.configure_module_resolution(tsconfig, workspace, snapshot.as_ref(), root);
    }
    Ok(Some(PreparedPlaywrightRules {
        snapshot,
        selections: prepared_selections,
        fact_plan,
        warnings,
        failures,
    }))
}

mod add_settings;
#[cfg(test)]
mod tests;
use add_settings::add_settings_facts;
mod partial;

pub(super) use partial::PreparationOptions;
