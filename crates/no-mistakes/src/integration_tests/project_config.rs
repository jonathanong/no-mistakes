use super::test_config;
use super::types::{ConfigProject, Framework};
use crate::codebase::ts_resolver::{ImportResolution, TsConfig};
use crate::config::v2::schema::StringOrList;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::Path;

mod discovery;
mod globs;
mod json;
mod parse;
pub(crate) use discovery::discovered_config_paths;
pub(crate) use globs::{build_globset, prefix_globs};
pub(super) use json::load_vitest_json_projects;
pub(in crate::integration_tests) use parse::load_config_projects_from_program;

pub(crate) fn load_projects(
    root: &Path,
    framework: Framework,
    configs: Option<&StringOrList>,
) -> Result<Vec<ConfigProject>> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible_paths = snapshot.paths_for(root);
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, root, &visible_paths)?;
    load_projects_from_visible(root, framework, configs, &visible_paths, &tsconfig)
}

pub(crate) fn load_projects_from_visible(
    root: &Path,
    framework: Framework,
    configs: Option<&StringOrList>,
    visible_paths: &[std::path::PathBuf],
    tsconfig: &TsConfig,
) -> Result<Vec<ConfigProject>> {
    let catalog =
        crate::codebase::ts_resolver::TsConfigCatalog::forced(root, tsconfig.clone(), None);
    load_projects_from_visible_with_catalog(root, framework, configs, visible_paths, &catalog)
}

pub(crate) fn load_projects_from_visible_with_catalog(
    root: &Path,
    framework: Framework,
    configs: Option<&StringOrList>,
    visible_paths: &[std::path::PathBuf],
    tsconfig_catalog: &crate::codebase::ts_resolver::TsConfigCatalog,
) -> Result<Vec<ConfigProject>> {
    // Keep every path used for config-relative glob prefixing in the same
    // lexical form. Callers may pass roots containing `..`, while the frozen
    // visible inventory is canonicalized during discovery.
    let normalized_root = crate::codebase::ts_resolver::normalize_path(root);
    let root = normalized_root.as_path();
    let mut visible_files = visible_paths
        .iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(path))
        .collect::<HashSet<_>>();
    let config_values = if let Some(configs) = configs {
        let config_values =
            globs::expand_explicit_config_values(root, &configs.values(), &visible_files);
        visible_files.extend(
            config_values
                .iter()
                .map(|raw| crate::codebase::ts_resolver::normalize_path(&root.join(raw))),
        );
        config_values
    } else {
        discovered_config_paths(root, framework, visible_paths)
    };
    let mut projects = Vec::new();
    let resolver = crate::codebase::ts_resolver::ScopedImportResolver::from_visible(
        tsconfig_catalog,
        &visible_files,
    );
    for raw in config_values {
        let path = crate::codebase::ts_resolver::normalize_path(&root.join(&raw));
        if !visible_files.contains(&path) {
            anyhow::bail!(
                "{} config does not exist: {}",
                framework.as_str(),
                path.display()
            );
        }
        let source = crate::integration_tests::runner_config::read_request_source(&path)
            .with_context(|| {
                format!(
                    "{} config does not exist: {}",
                    framework.as_str(),
                    path.display()
                )
            })?;
        let config_dir = path.parent().unwrap_or(root);
        projects.extend(load_config_projects_inner(
            ConfigProjectInput {
                root,
                framework,
                raw: &raw,
                path: &path,
                source: &source,
                config_dir,
                resolver: &resolver,
            },
            Some(&visible_files),
        )?);
    }
    Ok(projects)
}

pub(super) struct ConfigProjectInput<'a> {
    pub(super) root: &'a Path,
    pub(super) framework: Framework,
    pub(super) raw: &'a str,
    pub(super) path: &'a Path,
    pub(super) source: &'a str,
    pub(super) config_dir: &'a Path,
    pub(super) resolver: &'a dyn ImportResolution,
}

pub(super) fn load_config_projects_inner(
    input: ConfigProjectInput<'_>,
    visible_files: Option<&HashSet<std::path::PathBuf>>,
) -> Result<Vec<ConfigProject>> {
    let ConfigProjectInput {
        root,
        framework,
        raw,
        path,
        source,
        config_dir,
        resolver,
    } = input;
    if !framework.has_js_runner_config() {
        return Ok(Vec::new());
    }
    if framework == Framework::Jest {
        return Ok(vec![test_config::jest::config_project(
            root,
            raw,
            config_dir,
            source,
            visible_files,
        )?]);
    }
    if framework == Framework::Vitest
        && path.extension().and_then(|value| value.to_str()) == Some("json")
    {
        anyhow::ensure!(
            test_config::vitest::is_vitest_project_array_path(path),
            "unsupported Vitest JSON config filename: {}; use vitest.workspace.json or vitest.projects.json",
            path.display()
        );
        return json::load_vitest_json_projects(input);
    }
    crate::integration_tests::runner_config::with_program(path, source, |program, _| {
        load_config_projects_from_program(
            ConfigProjectInput {
                root,
                framework,
                raw,
                path,
                source,
                config_dir,
                resolver,
            },
            program,
            visible_files,
        )
    })?
}

#[cfg(test)]
mod tests;
