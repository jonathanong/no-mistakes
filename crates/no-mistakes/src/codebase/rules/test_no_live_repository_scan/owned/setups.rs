use super::{consumers::Consumed, imports::ImportedWrites};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::rules::PreparedVitestProjectCatalog;
use crate::codebase::test_discovery::ProjectTestFilter;
use crate::fx::FxHashSet;
use crate::integration_tests::types::VitestSetupField;
use std::path::{Path, PathBuf};

type Project = (ProjectTestFilter, Vec<PathBuf>);

pub(super) struct ConfiguredSetups {
    root: PathBuf,
    projects: Vec<Project>,
    known: bool,
}

impl ConfiguredSetups {
    pub(super) fn new(root: &Path, catalog: Option<&PreparedVitestProjectCatalog>) -> Self {
        let projects = projects(catalog);
        Self {
            root: root.to_path_buf(),
            known: catalog.is_none() || projects.is_some(),
            projects: projects.unwrap_or_default(),
        }
    }

    pub(super) fn setup_files(&self) -> Option<FxHashSet<PathBuf>> {
        self.known.then(|| {
            self.projects
                .iter()
                .flat_map(|(_, files)| files.iter().cloned())
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
        for (filter, setups) in &self.projects {
            if filter.is_match(&relative) {
                matched = true;
                if !setups.iter().any(|setup| {
                    imports.module_owns_before(setup, file, name, facts, consumed, fixture_dirs)
                }) {
                    return false;
                }
            }
        }
        matched
    }
}

fn projects(catalog: Option<&PreparedVitestProjectCatalog>) -> Option<Vec<Project>> {
    catalog?
        .merged_projects()
        .ok()?
        .into_iter()
        .map(|project| {
            let filter = ProjectTestFilter::from_project_ref(&project).ok()?;
            project.matcher_warnings.is_empty().then_some(())?;
            let setups = project
                .vitest_setup
                .into_iter()
                .filter(|setup| {
                    !setup.config_extends_provenance && setup.field == VitestSetupField::SetupFiles
                })
                .map(|setup| setup.resolved_path)
                .collect::<Option<Vec<_>>>()?;
            Some((filter, setups))
        })
        .collect()
}
