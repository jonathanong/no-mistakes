use super::{consumers::Consumed, imports::ImportedWrites};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::rules::PreparedVitestProjectCatalog;
use crate::codebase::test_discovery::ProjectTestFilter;
use crate::fx::FxHashSet;
use crate::integration_tests::types::VitestSetupField;
use std::path::{Path, PathBuf};

#[derive(Clone)]
struct Project {
    filter: ProjectTestFilter,
    setups: Vec<PathBuf>,
    config: Option<PathBuf>,
}

#[derive(Clone)]
pub(in super::super) struct ConfiguredSetups {
    root: PathBuf,
    projects: Vec<Project>,
    known: bool,
    configs_known: bool,
}

impl ConfiguredSetups {
    pub(in super::super) fn new(
        root: &Path,
        catalog: Option<&PreparedVitestProjectCatalog>,
        root_files: &[PathBuf],
    ) -> Self {
        let projects = projects(catalog, root, root_files);
        let configs_known = projects.as_ref().is_some_and(|projects| {
            catalog.is_some() && projects.iter().all(|project| project.config.is_some())
        });
        Self {
            root: root.to_path_buf(),
            known: catalog.is_none() || projects.is_some(),
            projects: projects.unwrap_or_default(),
            configs_known,
        }
    }

    pub(in super::super) fn setup_files(&self) -> Option<FxHashSet<PathBuf>> {
        self.known.then(|| {
            self.projects
                .iter()
                .flat_map(|project| project.setups.iter().cloned())
                .collect()
        })
    }

    pub(in super::super) fn known(&self) -> bool {
        self.known
    }

    pub(in super::super) fn config_files(&self) -> Option<FxHashSet<PathBuf>> {
        self.configs_known.then(|| {
            self.projects
                .iter()
                .filter_map(|project| project.config.clone())
                .collect()
        })
    }

    pub(super) fn owns(
        &self,
        file: &Path,
        name: &str,
        facts: &dyn TsFactLookup,
        imports: &ImportedWrites<'_>,
        consumed: &Consumed,
        fixture_dirs: &[String],
    ) -> bool {
        let relative = crate::codebase::ts_source::relative_slash_path(&self.root, file);
        let mut matched = false;
        for project in &self.projects {
            if project.filter.is_match(&relative) {
                matched = true;
                if !project.setups.iter().any(|setup| {
                    imports.module_owns_before(setup, file, name, facts, consumed, fixture_dirs)
                }) {
                    return false;
                }
            }
        }
        matched
    }
}

fn projects(
    catalog: Option<&PreparedVitestProjectCatalog>,
    root: &Path,
    root_files: &[PathBuf],
) -> Option<Vec<Project>> {
    let all = catalog?.merged_projects().ok()?;
    let mut selected = Vec::new();
    for project in all {
        let filter = ProjectTestFilter::from_project_ref(&project).ok()?;
        project.matcher_warnings.is_empty().then_some(())?;
        if !root_files.iter().any(|file| {
            filter.is_match(&crate::codebase::ts_source::relative_slash_path(root, file))
        }) {
            continue;
        }
        let setups = project
            .vitest_setup
            .into_iter()
            .filter(|setup| {
                !setup.config_extends_provenance && setup.field == VitestSetupField::SetupFiles
            })
            .map(|setup| setup.resolved_path)
            .collect::<Option<Vec<_>>>()?;
        let config = project
            .config
            .map(|path| crate::codebase::ts_resolver::normalize_path(&root.join(path)));
        selected.push(Project {
            filter,
            setups,
            config,
        });
    }
    (!selected.is_empty()).then_some(selected)
}
