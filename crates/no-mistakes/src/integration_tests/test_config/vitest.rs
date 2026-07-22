use super::shared;
use crate::codebase::ts_resolver::ImportResolution;
use crate::integration_tests::project_config::prefix_globs;
use crate::integration_tests::types::{ConfigProject, VitestSetupDependency};
use anyhow::Result;
use oxc_ast::ast::Program;
use std::path::{Path, PathBuf};

mod project_arrays;
#[cfg(test)]
pub(in crate::integration_tests) mod tests;

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
    /// A project config named by `test.projects` is a standalone config file.
    /// Its relative settings are therefore based on that file, not the
    /// aggregate config that happened to reference it.
    pub(super) config_base: Option<PathBuf>,
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
    let Some(root_object) = shared::default_export_object(program, &bindings) else {
        return Ok(Vec::new());
    };
    let root_options = project_arrays::root_options(program, root_object, source, path, resolver)?;
    let project_options =
        project_arrays::project_options(program, root_object, source, path, root, resolver)?;
    let mut projects = Vec::new();
    if project_options.is_empty() {
        projects.push(to_project(config_dir, root, root_options, resolver));
        return Ok(projects);
    }

    for project_options in project_options {
        projects.push(to_project(
            config_dir,
            root,
            merge_options(&root_options, project_options),
            resolver,
        ));
    }
    Ok(projects)
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
        resolver,
    );
    ConfigProject {
        config: None,
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

fn resolve_setup_dependencies<'a>(
    dependencies: impl Iterator<Item = &'a mut VitestSetupDependency>,
    project_root: &Path,
    resolver: &dyn ImportResolution,
) {
    // `ImportResolver` takes an importing file, while Vitest resolves these
    // fields from the effective project root. A stable synthetic filename
    // makes its parent exactly that root without reading or executing config.
    let resolution_source = project_root.join(".no-mistakes-vitest-setup.ts");
    for dependency in dependencies {
        dependency.resolution_base = project_root.to_path_buf();
        dependency.resolved_path = dependency
            .specifier
            .as_deref()
            .and_then(|specifier| resolver.resolve(specifier, &resolution_source));
    }
}

fn merge_options(root: &Options, project: Options) -> Options {
    Options {
        name: project.name.or_else(|| root.name.clone()),
        root: project.root.or_else(|| root.root.clone()),
        include: project.include.or_else(|| root.include.clone()),
        exclude: combine(root.exclude.clone(), project.exclude),
        setup_files: project.setup_files.or_else(|| root.setup_files.clone()),
        global_setup: project.global_setup.or_else(|| root.global_setup.clone()),
        config_base: project.config_base.or_else(|| root.config_base.clone()),
    }
}

fn combine(left: Option<Vec<String>>, right: Option<Vec<String>>) -> Option<Vec<String>> {
    let mut values = left.unwrap_or_default();
    values.extend(right.unwrap_or_default());
    (!values.is_empty()).then_some(values)
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
