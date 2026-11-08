use super::imports::ImportedWrites;
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::rules::PreparedVitestProjectCatalog;
use crate::codebase::test_discovery::ProjectTestFilter;
use crate::integration_tests::types::VitestSetupField;
use std::path::{Path, PathBuf};

type Project = (ProjectTestFilter, Vec<PathBuf>);

pub(super) struct ConfiguredSetups {
    root: PathBuf,
    projects: Vec<Project>,
}

impl ConfiguredSetups {
    pub(super) fn new(root: &Path, catalog: Option<&PreparedVitestProjectCatalog>) -> Self {
        Self {
            root: root.to_path_buf(),
            projects: projects(catalog).unwrap_or_default(),
        }
    }

    pub(super) fn owns(
        &self,
        file: &Path,
        name: &str,
        facts: &dyn TsFactLookup,
        imports: &ImportedWrites<'_>,
    ) -> bool {
        let relative = crate::codebase::ts_source::relative_slash_path(&self.root, file);
        let mut matched = false;
        for (filter, setups) in &self.projects {
            if filter.is_match(&relative) {
                matched = true;
                if !setups
                    .iter()
                    .any(|setup| imports.module_owns_before(setup, file, name, facts))
                {
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
