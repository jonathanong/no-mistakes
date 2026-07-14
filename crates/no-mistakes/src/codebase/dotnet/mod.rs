mod config;
mod csharp;
mod csharp_strip;
mod project;
mod project_static;
mod types;

use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

pub(crate) use config::configured_projects;
pub(crate) use csharp::parse_csharp_file;
use project::parse_project;
pub(crate) use types::{DotnetConfigProject, DotnetFactMap, DotnetFileFacts, DotnetProjectFacts};

pub(crate) fn collect_dotnet_facts(
    root: &Path,
    all_files: &[PathBuf],
    projects: &[DotnetConfigProject],
) -> DotnetFactMap {
    #[cfg(any(test, feature = "test-instrumentation"))]
    test_support::record_fact_collection(root);
    if projects.is_empty() {
        return DotnetFactMap::default();
    }

    let project_facts: Vec<(Option<DotnetProjectFacts>, Vec<String>)> = projects
        .par_iter()
        .map(|project| parse_project(root, all_files, project))
        .collect();

    let mut facts = DotnetFactMap::default();
    for (project, warnings) in project_facts {
        facts.warnings.extend(warnings);
        let Some(project) = project else {
            continue;
        };
        facts
            .files_by_project
            .entry(project.project_path.clone())
            .or_default()
            .extend(project.compile_files.iter().cloned());
        facts.projects.insert(project.project_path.clone(), project);
    }

    let project_by_file = project_index(&facts.projects);
    let cs_files: Vec<PathBuf> = all_files
        .iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("cs"))
        .filter(|path| project_by_file.contains_key(*path))
        .cloned()
        .collect();

    let mut file_facts: Vec<DotnetFileFacts> = cs_files
        .par_iter()
        .filter_map(|path| parse_csharp_file(path, project_by_file.get(path).cloned()))
        .collect();
    file_facts.sort_by(|a, b| a.path.cmp(&b.path));

    for file in file_facts {
        index_file(&mut facts, file);
    }

    facts
}

fn index_file(facts: &mut DotnetFactMap, file: DotnetFileFacts) {
    if let Some(project) = &file.project {
        facts
            .files_by_project
            .entry(project.clone())
            .or_default()
            .insert(file.path.clone());
    }
    if let Some(namespace) = &file.namespace {
        facts
            .files_by_namespace
            .entry(namespace.clone())
            .or_default()
            .insert(file.path.clone());
    }
    for declaration in &file.declarations {
        facts
            .declarations
            .entry(declaration.clone())
            .or_default()
            .insert(file.path.clone());
        if let Some(namespace) = &file.namespace {
            facts
                .declarations
                .entry(format!("{namespace}.{declaration}"))
                .or_default()
                .insert(file.path.clone());
        }
    }
    facts.files.insert(file.path.clone(), file);
}

fn project_index(projects: &BTreeMap<PathBuf, DotnetProjectFacts>) -> HashMap<PathBuf, PathBuf> {
    let mut index = HashMap::new();
    for project in projects.values() {
        for file in &project.compile_files {
            index.insert(file.clone(), project.project_path.clone());
        }
    }
    index
}

pub(super) fn normalize_path(path: &Path) -> PathBuf {
    crate::codebase::ts_resolver::normalize_path(path)
}

pub(super) fn msbuild_path(raw: &str) -> PathBuf {
    PathBuf::from(raw.replace('\\', "/"))
}

#[cfg(any(test, feature = "test-instrumentation"))]
pub mod test_support;
#[cfg(test)]
mod tests;
