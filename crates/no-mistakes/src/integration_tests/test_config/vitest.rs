use super::shared;
use crate::ast;
use crate::codebase::ts_resolver::TsConfig;
use crate::integration_tests::project_config::prefix_globs;
use crate::integration_tests::types::ConfigProject;
use anyhow::Result;
use oxc_ast::ast::Program;
use std::path::Path;

mod project_arrays;

const DEFAULT_EXTENSIONS: &[&str] = &[
    "js", "jsx", "ts", "tsx", "mjs", "mjsx", "mts", "mtsx", "cjs", "cjsx", "cts", "ctsx",
];

#[derive(Default, Clone)]
pub(super) struct Options {
    pub(super) name: Option<String>,
    pub(super) root: Option<String>,
    pub(super) include: Option<Vec<String>>,
    pub(super) exclude: Option<Vec<String>>,
}

pub(in crate::integration_tests) fn parse_from_path(
    source: &str,
    path: &Path,
    config_dir: &Path,
    root: &Path,
    tsconfig: &TsConfig,
) -> Result<Vec<ConfigProject>> {
    ast::with_program(path, source, |program, source| {
        parse_program(program, source, path, config_dir, root, tsconfig)
    })?
}

fn parse_program(
    program: &Program<'_>,
    source: &str,
    path: &Path,
    config_dir: &Path,
    root: &Path,
    tsconfig: &TsConfig,
) -> Result<Vec<ConfigProject>> {
    let bindings = shared::top_level_object_bindings(program);
    let Some(root_object) = shared::default_export_object(program, &bindings) else {
        return Ok(Vec::new());
    };
    let root_options = project_arrays::root_options(program, root_object, source, path, tsconfig)?;
    let project_options =
        project_arrays::project_options(program, root_object, source, path, root, tsconfig)?;
    let mut projects = Vec::new();
    if project_options.is_empty() {
        projects.push(to_project(config_dir, root, root_options));
        return Ok(projects);
    }

    for project_options in project_options {
        projects.push(to_project(
            config_dir,
            root,
            merge_options(&root_options, project_options),
        ));
    }
    Ok(projects)
}

fn to_project(config_dir: &Path, root: &Path, options: Options) -> ConfigProject {
    let include = options.include.unwrap_or_else(default_include);
    let config_dir = options
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
    ConfigProject {
        config: None,
        policy_name: options.name.clone(),
        runner_project_arg: options.name,
        include: prefix_globs(root, &config_dir, &include),
        exclude: prefix_globs(root, &config_dir, &options.exclude.unwrap_or_default()),
    }
}

fn merge_options(root: &Options, project: Options) -> Options {
    Options {
        name: project.name.or_else(|| root.name.clone()),
        root: project.root.or_else(|| root.root.clone()),
        include: project.include.or_else(|| root.include.clone()),
        exclude: combine(root.exclude.clone(), project.exclude),
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
