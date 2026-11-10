//! Why a Vitest catalog did not yield test roots. One bad project fails all of them.
use super::Project;
use crate::codebase::check_outcome::CheckInputFailure;
use crate::codebase::rules::PreparedVitestProjectCatalog;
use crate::codebase::test_discovery::ProjectTestFilter;
use crate::codebase::ts_resolver::normalize_path;
use crate::integration_tests::types::{ConfigProject, VitestSetupField};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub(crate) enum SetupGap {
    /// Matching projects left at least one `setupFiles` entry unresolved.
    Unresolved(Vec<(PathBuf, u32)>),
    /// The catalog could not be read, or no project matches the selected tests.
    Unreadable(PathBuf),
}

pub(super) fn select(
    catalog: &PreparedVitestProjectCatalog,
    root: &Path,
    root_files: &[PathBuf],
) -> Result<Vec<Project>, SetupGap> {
    let projects = catalog
        .merged_projects()
        .map_err(|error| SetupGap::Unreadable(config_from_error(&error, root)))?;
    let mut config_gap: Option<PathBuf> = None;
    let mut setup_holes = Vec::new();
    let mut selected = Vec::new();
    let mut configs = Vec::new();
    for project in projects {
        let config = project_config(root, &project);
        configs.push(config.clone());
        let Ok(filter) = ProjectTestFilter::from_project_ref(&project) else {
            config_gap = earlier(config_gap, config);
            continue;
        };
        if !project.matcher_warnings.is_empty() {
            config_gap = earlier(config_gap, config);
            continue;
        }
        if !matches_root(&filter, root, root_files) {
            continue;
        }
        match project_setups(&project) {
            Ok((setups, global_setups)) => selected.push(Project {
                filter,
                setups,
                global_setups,
                config: declared_config(root, &project),
            }),
            Err(holes) => setup_holes.extend(holes),
        }
    }
    if let Some(file) = config_gap {
        return Err(SetupGap::Unreadable(file));
    }
    if !setup_holes.is_empty() {
        setup_holes.sort();
        setup_holes.dedup();
        return Err(SetupGap::Unresolved(setup_holes));
    }
    if selected.is_empty() {
        configs.sort();
        let file = configs
            .into_iter()
            .next()
            .unwrap_or_else(|| normalize_path(&root.join("vitest.config.ts")));
        return Err(SetupGap::Unreadable(file));
    }
    Ok(selected)
}

fn earlier(current: Option<PathBuf>, next: PathBuf) -> Option<PathBuf> {
    Some(match current {
        Some(path) if path <= next => path,
        _ => next,
    })
}

fn matches_root(filter: &ProjectTestFilter, root: &Path, root_files: &[PathBuf]) -> bool {
    root_files
        .iter()
        .any(|file| filter.is_match(&crate::codebase::ts_source::relative_slash_path(root, file)))
}

fn project_setups(
    project: &ConfigProject,
) -> Result<(Vec<PathBuf>, Vec<PathBuf>), Vec<(PathBuf, u32)>> {
    let global_setups = project
        .vitest_setup
        .iter()
        .filter(|setup| {
            !setup.config_extends_provenance && setup.field == VitestSetupField::GlobalSetup
        })
        .filter_map(|setup| setup.resolved_path.clone())
        .collect();
    let mut holes = Vec::new();
    let mut setups = Vec::new();
    for setup in &project.vitest_setup {
        if setup.config_extends_provenance || setup.field != VitestSetupField::SetupFiles {
            continue;
        }
        match &setup.resolved_path {
            Some(path) => setups.push(path.clone()),
            None => holes.push((
                normalize_path(&setup.declaration_path),
                setup.declaration_line.max(1),
            )),
        }
    }
    if holes.is_empty() {
        Ok((setups, global_setups))
    } else {
        Err(holes)
    }
}

fn declared_config(root: &Path, project: &ConfigProject) -> Option<PathBuf> {
    project
        .config
        .as_ref()
        .map(|path| normalize_path(&root.join(path)))
}

fn project_config(root: &Path, project: &ConfigProject) -> PathBuf {
    declared_config(root, project).unwrap_or_else(|| normalize_path(&root.join("vitest.config.ts")))
}

fn config_from_error(error: &anyhow::Error, root: &Path) -> PathBuf {
    error
        .downcast_ref::<CheckInputFailure>()
        .and_then(|failure| failure.file.clone())
        .map(|file| normalize_path(&file))
        .unwrap_or_else(|| normalize_path(&root.join("vitest.config.ts")))
}
