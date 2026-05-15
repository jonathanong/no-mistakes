use super::ast_nav::{default_export_object, project_objects, top_level_object_bindings};
use super::merge::{merge_project, parse_options};
use super::types::{ParsedOptions, PlaywrightConfig};
use crate::ast;
use anyhow::Result;
use oxc_ast::ast::Program;
use std::path::Path;

#[cfg(test)]
pub fn parse(source: &str, config_dir: &Path) -> Result<PlaywrightConfig> {
    parse_from_path(source, Path::new("playwright.config.ts"), config_dir)
}

pub fn parse_from_path(source: &str, path: &Path, config_dir: &Path) -> Result<PlaywrightConfig> {
    ast::with_program(path, source, |program, source| {
        parse_program(program, source, config_dir)
    })?
}

fn parse_program(
    program: &Program<'_>,
    source: &str,
    config_dir: &Path,
) -> Result<PlaywrightConfig> {
    let bindings = top_level_object_bindings(program);
    let Some(root_object) = default_export_object(program) else {
        return Ok(PlaywrightConfig {
            name: None,
            projects: vec![merge_project(config_dir, &ParsedOptions::default(), None)],
        });
    };
    let root_options = parse_options(root_object, source, &bindings)?;
    let project_objects = project_objects(root_object);

    if project_objects.is_empty() {
        return Ok(PlaywrightConfig {
            name: root_options.name.clone(),
            projects: vec![merge_project(config_dir, &root_options, None)],
        });
    }

    let mut projects = Vec::new();
    for project_object in project_objects {
        projects.push(merge_project(
            config_dir,
            &root_options,
            Some(parse_options(project_object, source, &bindings)?),
        ));
    }

    Ok(PlaywrightConfig {
        name: root_options.name,
        projects,
    })
}
