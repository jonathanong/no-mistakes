use super::*;
use crate::codebase::check_outcome::CheckFailure;
use crate::playwright::playwright_config::{self, PlaywrightConfig};
use std::path::PathBuf;

pub(super) fn selection_failures(
    selection: &super::super::selection::RuleSelection,
    file: Option<&PathBuf>,
    error: &anyhow::Error,
) -> Vec<CheckFailure> {
    [
        (selection.coverage, super::super::PLAYWRIGHT_COVERAGE),
        (
            selection.unique_test_ids,
            super::super::PLAYWRIGHT_UNIQUE_TEST_IDS,
        ),
        (
            selection.unique_html_ids,
            super::super::PLAYWRIGHT_UNIQUE_HTML_IDS,
        ),
        (
            selection.prefer_test_id_locators,
            super::super::PLAYWRIGHT_PREFER_TEST_ID_LOCATORS,
        ),
    ]
    .into_iter()
    .filter(|(enabled, _)| *enabled)
    .map(|(_, consumer)| CheckFailure {
        consumer: consumer.to_string(),
        file: file.cloned(),
        property: error
            .to_string()
            .contains("testDir")
            .then(|| "testDir".to_string()),
        message: format!("{error:#}"),
    })
    .collect()
}

type LoadedConfigs = Vec<(PathBuf, PlaywrightConfig)>;
pub(super) type ConfigFailures = Vec<(PathBuf, Option<String>, Option<String>, anyhow::Error)>;

pub(super) fn load_configs(
    root: &Path,
    paths: &[PathBuf],
    snapshot: &VisiblePathSnapshot,
    partial: bool,
) -> Result<(LoadedConfigs, ConfigFailures)> {
    if !partial {
        return Ok((
            playwright_config::load_configs_with_sources(
                root,
                paths,
                Some(&snapshot.source_store_for(root)),
            )?,
            Vec::new(),
        ));
    }
    let mut loaded = Vec::new();
    let mut failures = Vec::new();
    for path in paths {
        match playwright_config::load_partial_with_sources(
            root,
            path,
            &snapshot.source_store_for(root),
        ) {
            Ok((config, errors)) => {
                let config_name = config.name.clone();
                loaded.push((
                    crate::codebase::ts_resolver::normalize_path(&root.join(path)),
                    config,
                ));
                failures.extend(
                    errors.into_iter().map(|(project, error)| {
                        (path.clone(), project, config_name.clone(), error)
                    }),
                );
            }
            Err(error) => failures.push((
                path.clone(),
                None,
                playwright_config::config_name_with_sources(path, &snapshot.source_store_for(root)),
                error,
            )),
        }
    }
    Ok((loaded, failures))
}

pub(super) fn apply_failures(
    selection: &super::super::selection::RuleSelection,
    settings: &mut config::Settings,
    loaded: &LoadedConfigs,
    errors: &ConfigFailures,
    failures: &mut Vec<CheckFailure>,
) -> bool {
    let previous_failures = failures.len();
    for (path, project, config_name, error) in errors {
        let unrelated = project
            .as_deref()
            .zip(settings.project.as_deref())
            .is_some_and(|(failed, selected)| {
                failed != selected
                    && loaded.iter().any(|(loaded_path, config)| {
                        loaded_path == path
                            && config.name.as_deref() != Some(selected)
                            && config
                                .projects
                                .iter()
                                .any(|project| project.name.as_deref() == Some(selected))
                    })
            });
        let unrelated_config = config_name
            .as_deref()
            .zip(settings.project.as_deref())
            .is_some_and(|(failed, selected)| {
                failed != selected
                    && loaded
                        .iter()
                        .any(|(_, config)| config.name.as_deref() == Some(selected))
            });
        if !unrelated && !unrelated_config {
            failures.extend(selection_failures(selection, Some(path), error));
        }
    }
    if failures.len() != previous_failures {
        return false;
    }
    settings
        .playwright_configs
        .retain(|path| loaded.iter().any(|(loaded_path, _)| loaded_path == path));
    if settings.playwright_configs.is_empty()
        || !loaded.iter().any(|(path, config)| {
            settings.playwright_configs.contains(path) && !config.projects.is_empty()
        })
    {
        return false;
    }
    settings.project.as_deref().is_none_or(|name| {
        loaded.iter().any(|(_, config)| {
            config.name.as_deref() == Some(name)
                || config
                    .projects
                    .iter()
                    .any(|project| project.name.as_deref() == Some(name))
        })
    })
}

pub(super) fn selection_consumes(
    selection: &super::super::selection::RuleSelection,
    consumer: &str,
) -> bool {
    match consumer {
        super::super::PLAYWRIGHT_COVERAGE => selection.coverage,
        super::super::PLAYWRIGHT_UNIQUE_TEST_IDS => selection.unique_test_ids,
        super::super::PLAYWRIGHT_UNIQUE_HTML_IDS => selection.unique_html_ids,
        super::super::PLAYWRIGHT_PREFER_TEST_ID_LOCATORS => selection.prefer_test_id_locators,
        _ => false,
    }
}

pub(super) fn normalize_settings(root: &Path, mut settings: config::Settings) -> config::Settings {
    for path in &mut settings.playwright_configs {
        *path = crate::codebase::ts_resolver::normalize_path(&root.join(&*path));
    }
    settings
}

#[derive(Default)]
pub(in crate::playwright::rules) struct PreparationOptions {
    pub(in crate::playwright::rules) tsconfig_catalog:
        Option<Arc<crate::codebase::ts_resolver::TsConfigCatalog>>,
    pub(in crate::playwright::rules) partial: bool,
}
