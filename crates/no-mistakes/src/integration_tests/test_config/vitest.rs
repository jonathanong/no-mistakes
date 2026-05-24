use super::shared;
use crate::ast;
use crate::codebase::ts_resolver::TsConfig;
use crate::integration_tests::project_config::prefix_globs;
use crate::integration_tests::types::ConfigProject;
use anyhow::Result;
use oxc_ast::ast::{ObjectExpression, Program};
use std::path::Path;

mod project_arrays;

const DEFAULT_INCLUDE: &[&str] = &[
    "**/*.{test,spec}.?(c|m)[jt]s?(x)",
    "**/__tests__/**/*.?(c|m)[jt]s?(x)",
];

#[derive(Default, Clone)]
pub(super) struct Options {
    pub(super) name: Option<String>,
    pub(super) include: Option<Vec<String>>,
    pub(super) exclude: Option<Vec<String>>,
}

pub(in crate::integration_tests) fn parse_from_path(
    source: &str,
    path: &Path,
    config_dir: &Path,
    root: &Path,
) -> Result<Vec<ConfigProject>> {
    ast::with_program(path, source, |program, source| {
        let tsconfig = crate::integration_tests::project_config::resolve_tsconfig(root)
            .unwrap_or_else(|_| crate::integration_tests::tsconfig_without_config(root));
        parse_program(program, source, path, config_dir, root, &tsconfig)
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
    let test_object =
        shared::property_object(root_object, "test", &bindings).unwrap_or(root_object);
    let root_options = parse_options(test_object, source)?;
    let project_options =
        project_arrays::project_options(program, test_object, source, path, root, tsconfig);
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
    let include = options.include.unwrap_or_else(|| {
        DEFAULT_INCLUDE
            .iter()
            .map(|glob| glob.to_string())
            .collect()
    });
    ConfigProject {
        config: None,
        name: options.name,
        include: prefix_globs(root, config_dir, &include),
        exclude: prefix_globs(root, config_dir, &options.exclude.unwrap_or_default()),
    }
}

fn merge_options(root: &Options, project: Options) -> Options {
    Options {
        name: project.name.or_else(|| root.name.clone()),
        include: project.include.or_else(|| root.include.clone()),
        exclude: combine(root.exclude.clone(), project.exclude),
    }
}

fn combine(left: Option<Vec<String>>, right: Option<Vec<String>>) -> Option<Vec<String>> {
    let mut values = left.unwrap_or_default();
    values.extend(right.unwrap_or_default());
    (!values.is_empty()).then_some(values)
}

pub(super) fn parse_options(object: &ObjectExpression<'_>, source: &str) -> Result<Options> {
    Ok(Options {
        name: shared::property_expression(object, "name")
            .and_then(|value| shared::optional_string(value, source)),
        include: string_array_property(object, source, "include")?,
        exclude: string_array_property(object, source, "exclude")?,
    })
}

pub(super) fn parse_partial_options(object: &ObjectExpression<'_>, source: &str) -> Options {
    Options {
        name: shared::property_expression(object, "name")
            .and_then(|value| shared::optional_string(value, source)),
        include: string_array_property(object, source, "include")
            .ok()
            .flatten(),
        exclude: string_array_property(object, source, "exclude")
            .ok()
            .flatten(),
    }
}

fn string_array_property(
    object: &ObjectExpression<'_>,
    source: &str,
    name: &str,
) -> Result<Option<Vec<String>>> {
    shared::property_expression(object, name)
        .map(|value| {
            let values = shared::inferred_string_or_array(value, source, name)?;
            if values.is_empty() && name != "exclude" {
                anyhow::bail!("expected string literal or string array for {name}");
            }
            Ok(values)
        })
        .transpose()
}
