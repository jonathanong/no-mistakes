use super::test_config;
use super::types::{ConfigProject, Framework};
use crate::codebase::ts_resolver::TsConfig;
use crate::config::v2::schema::StringOrList;
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::Path;

mod discovery;
mod globs;
pub(crate) use discovery::discovered_config_paths;
pub(crate) use globs::{build_globset, prefix_globs};

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
        let config_values = configs.values();
        // Explicit runner configs are authoritative even when Git ignores
        // them. Authorize only those config paths in this local parse view;
        // ignored helpers remain outside the frozen visible file set.
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
                tsconfig,
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
    pub(super) tsconfig: &'a TsConfig,
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
        tsconfig,
    } = input;
    if matches!(framework, Framework::Dotnet | Framework::Swift) {
        return Ok(Vec::new());
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
                tsconfig,
            },
            program,
            visible_files,
        )
    })?
}

pub(super) fn load_config_projects_from_program(
    input: ConfigProjectInput<'_>,
    program: &oxc_ast::ast::Program<'_>,
    visible_files: Option<&HashSet<std::path::PathBuf>>,
) -> Result<Vec<ConfigProject>> {
    let ConfigProjectInput {
        root,
        framework,
        raw,
        path,
        source,
        config_dir,
        tsconfig,
    } = input;
    match framework {
        Framework::Dotnet => Ok(Vec::new()),
        Framework::Playwright => {
            let parsed = test_config::playwright::parse_program(
                program,
                source,
                path,
                config_dir,
                tsconfig,
                visible_files,
            )?;
            Ok(parsed.into_projects(root, raw))
        }
        Framework::Vitest => {
            let parsed = test_config::vitest::parse_program(
                program,
                source,
                path,
                config_dir,
                root,
                tsconfig,
                visible_files,
            )?;
            Ok(parsed
                .into_iter()
                .map(|mut project| {
                    project.config = Some(raw.to_string());
                    project
                })
                .collect())
        }
        Framework::Swift => Ok(Vec::new()),
    }
}

#[cfg(test)]
mod tests;
