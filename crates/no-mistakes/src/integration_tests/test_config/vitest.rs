use super::shared;
use crate::codebase::ts_resolver::ImportResolution;
use crate::integration_tests::project_config::prefix_globs;
use crate::integration_tests::types::{ConfigProject, VitestSetupDependency};
use anyhow::Result;
use oxc_ast::ast::Program;
use std::path::{Path, PathBuf};

mod json_projects;
mod merge;
mod project_arrays;
pub(crate) mod setup_resolution;
#[cfg(test)]
pub(in crate::integration_tests) mod tests;

use merge::merge_options;
use setup_resolution::resolve_setup_dependencies;

const DEFAULT_EXTENSIONS: &[&str] = &[
    "js", "jsx", "ts", "tsx", "mjs", "mjsx", "mts", "mtsx", "cjs", "cjsx", "cts", "ctsx",
];

#[derive(Default, Clone)]
pub(super) struct Options {
    pub(super) name: Option<String>,
    pub(super) root: Option<String>,
    pub(super) include: Option<Vec<String>>,
    pub(super) exclude: Option<Vec<String>>,
    pub(super) setup_files: Option<Vec<VitestSetupDependency>>,
    pub(super) global_setup: Option<Vec<VitestSetupDependency>>,
    /// A nested `test` object owns setup fields, including when it arrives
    /// through a supported static object spread.
    pub(super) nested_test_scope: bool,
    /// Whether an inline project inherits root setup fields, opts out, or
    /// inherits setup fields from another static config source.
    pub(super) extends: Option<Extends>,
    /// A config named directly by `test.projects` is independent of the
    /// aggregate config that referenced it.
    pub(super) standalone_config: bool,
    /// Canonical path of a standalone `test.projects` config, retained only
    /// while parsing so glob negations can remove the matching entry.
    pub(super) standalone_config_path: Option<PathBuf>,
    /// A project config named by `test.projects` is a standalone config file.
    /// Its relative settings are therefore based on that file, not the
    /// aggregate config that happened to reference it.
    pub(super) config_base: Option<PathBuf>,
}

#[derive(Clone, PartialEq, Eq)]
pub(super) enum Extends {
    False,
    True,
    Config(String),
}

pub(in crate::integration_tests) fn parse_program_with_resolver(
    program: &Program<'_>,
    source: &str,
    path: &Path,
    config_dir: &Path,
    root: &Path,
    resolver: &dyn ImportResolution,
) -> Result<Vec<ConfigProject>> {
    let bindings = shared::top_level_object_bindings(program);
    let root_object = shared::default_export_object(program, &bindings);
    let workspace_options = (root_object.is_none() && is_vitest_project_array_path(path))
        .then(|| project_arrays::workspace_options(program, source, path, resolver))
        .transpose()?
        .unwrap_or_default();
    if root_object.is_none() {
        return Ok(workspace_options
            .into_iter()
            .map(|options| to_project(config_dir, root, options, resolver))
            .collect());
    }
    let root_object = root_object.expect("workspace branch returns when config object is absent");
    let root_options = project_arrays::root_options(program, root_object, source, path, resolver)?;
    let project_options =
        project_arrays::project_options(program, root_object, source, path, resolver)?;
    let mut projects = Vec::new();
    if project_options.is_empty() {
        projects.push(to_project(config_dir, root, root_options, resolver));
        return Ok(projects);
    }

    for project_options in project_options {
        let options = if project_options.standalone_config {
            project_options
        } else {
            merge_options(&root_options, project_options)
        };
        projects.push(to_project(config_dir, root, options, resolver));
    }
    Ok(projects)
}

pub(crate) fn is_vitest_project_array_path(path: &Path) -> bool {
    const EXTENSIONS: &[&str] = &["mts", "ts", "mjs", "js", "cjs", "cts", "json"];
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    EXTENSIONS.contains(&extension)
        && path.file_stem().is_some_and(|stem| {
            matches!(stem.to_str(), Some("vitest.workspace" | "vitest.projects"))
        })
}

pub(in crate::integration_tests) fn parse_json_with_resolver(
    source: &str,
    path: &Path,
    config_dir: &Path,
    root: &Path,
    resolver: &dyn ImportResolution,
) -> Result<Vec<ConfigProject>> {
    json_projects::parse(source, path, config_dir, root, resolver)
}

fn to_project(
    config_dir: &Path,
    root: &Path,
    mut options: Options,
    resolver: &dyn ImportResolution,
) -> ConfigProject {
    let config_dir = options.config_base.as_deref().unwrap_or(config_dir);
    let include = options.include.unwrap_or_else(default_include);
    let project_root = options
        .root
        .as_deref()
        .map(|project_root| {
            let project_root = Path::new(project_root);
            if project_root.is_absolute() {
                project_root.to_path_buf()
            } else {
                config_dir.join(project_root)
            }
        })
        .unwrap_or_else(|| config_dir.to_path_buf());
    let project_root = crate::codebase::ts_resolver::normalize_path(&project_root);
    resolve_setup_dependencies(
        options
            .setup_files
            .iter_mut()
            .chain(options.global_setup.iter_mut())
            .flatten(),
        &project_root,
        root,
        resolver,
    );
    if let Some(setups) = options.setup_files.as_mut() {
        merge::dedupe_resolved_setups(setups);
    }
    if let Some(setups) = options.global_setup.as_mut() {
        merge::dedupe_resolved_setups(setups);
    }
    ConfigProject {
        config: None,
        workspace: false,
        policy_name: options.name.clone(),
        runner_project_arg: options.name,
        scope: Some(crate::codebase::ts_source::relative_slash_path(
            root,
            &project_root,
        )),
        include: prefix_globs(root, &project_root, &include),
        exclude: prefix_globs(root, &project_root, &options.exclude.unwrap_or_default()),
        vitest_setup: options
            .setup_files
            .into_iter()
            .flatten()
            .chain(options.global_setup.into_iter().flatten())
            .collect(),
    }
}

fn default_include() -> Vec<String> {
    let mut include = Vec::with_capacity(DEFAULT_EXTENSIONS.len() * 3);
    for ext in DEFAULT_EXTENSIONS {
        include.push(format!("**/*.test.{ext}"));
        include.push(format!("**/*.spec.{ext}"));
        include.push(format!("**/__tests__/**/*.{ext}"));
    }
    include
}
