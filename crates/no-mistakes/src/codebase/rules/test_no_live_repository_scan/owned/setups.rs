#[path = "setups_gap.rs"]
mod setups_gap;

use super::{consumers::Consumed, imports::ImportedWrites};
use crate::codebase::dependencies::graph::TsFactLookup;
use crate::codebase::rules::PreparedVitestProjectCatalog;
use crate::codebase::test_discovery::ProjectTestFilter;
use crate::fx::FxHashSet;
use std::path::{Path, PathBuf};

pub(super) use setups_gap::SetupGap;

pub(super) enum ReadSite<'a> {
    Call(&'a Path),
    Capture { test: &'a Path, module: &'a Path },
}

#[derive(Clone)]
struct Project {
    filter: ProjectTestFilter,
    setups: Vec<PathBuf>,
    global_setups: Vec<PathBuf>,
    config: Option<PathBuf>,
    concurrent: bool,
}

#[derive(Clone)]
pub(in super::super) struct ConfiguredSetups {
    root: PathBuf,
    projects: Vec<Project>,
    known: bool,
    configs_known: bool,
    /// Why test roots could not be named. Absent when `known` is true.
    gap: Option<SetupGap>,
}

impl ConfiguredSetups {
    pub(in super::super) fn new(
        root: &Path,
        catalog: Option<&PreparedVitestProjectCatalog>,
        root_files: &[PathBuf],
    ) -> Self {
        let (projects, gap) = match catalog {
            None => (Vec::new(), None),
            Some(catalog) => match setups_gap::select(catalog, root, root_files) {
                Ok(projects) => (projects, None),
                Err(gap) => (Vec::new(), Some(gap)),
            },
        };
        let known = gap.is_none();
        let configs_known = catalog.is_some()
            && !projects.is_empty()
            && projects.iter().all(|project| project.config.is_some());
        Self {
            root: root.to_path_buf(),
            known,
            projects,
            configs_known,
            gap,
        }
    }

    pub(super) fn root(&self) -> &Path {
        &self.root
    }

    pub(super) fn gap(&self) -> Option<&SetupGap> {
        self.gap.as_ref()
    }

    pub(in super::super) fn setup_files(&self) -> Option<FxHashSet<PathBuf>> {
        self.known.then(|| {
            self.projects
                .iter()
                .flat_map(|project| project.setups.iter().cloned())
                .collect()
        })
    }

    pub(in super::super) fn global_setup_files(&self) -> Option<FxHashSet<PathBuf>> {
        self.known.then(|| {
            self.projects
                .iter()
                .flat_map(|project| project.global_setups.iter().cloned())
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

    pub(super) fn concurrent(&self, file: &Path) -> bool {
        let relative = crate::codebase::ts_source::relative_slash_path(&self.root, file);
        self.projects
            .iter()
            .any(|project| project.concurrent && project.filter.is_match(&relative))
    }

    pub(super) fn has_cleanup(
        &self,
        file: &Path,
        name: &str,
        facts: &dyn TsFactLookup,
        imports: &ImportedWrites<'_>,
        consumed: &Consumed,
        fixture_dirs: &[String],
    ) -> bool {
        let relative = crate::codebase::ts_source::relative_slash_path(&self.root, file);
        self.projects
            .iter()
            .filter(|project| project.filter.is_match(&relative))
            .any(|project| {
                project
                    .setups
                    .iter()
                    .any(|setup| imports.has_cleanup(setup, name, facts, consumed, fixture_dirs))
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
        self.owns_at(
            ReadSite::Call(file),
            name,
            facts,
            imports,
            consumed,
            fixture_dirs,
        )
    }

    pub(super) fn owns_at(
        &self,
        site: ReadSite<'_>,
        name: &str,
        facts: &dyn TsFactLookup,
        imports: &ImportedWrites<'_>,
        consumed: &Consumed,
        fixture_dirs: &[String],
    ) -> bool {
        let (test, capture, captured) = match site {
            ReadSite::Call(file) => (file, file, false),
            ReadSite::Capture { test, module } => (test, module, true),
        };
        let relative = crate::codebase::ts_source::relative_slash_path(&self.root, test);
        let mut matched = false;
        for project in &self.projects {
            if project.filter.is_match(&relative) {
                matched = true;
                if captured
                    && (project
                        .config
                        .as_ref()
                        .is_some_and(|config| imports.loads_module(config, capture))
                        || project
                            .setups
                            .iter()
                            .any(|setup| imports.loads_module(setup, capture)))
                {
                    return false;
                }
                if !project.setups.iter().any(|setup| {
                    imports.module_owns_before(setup, capture, name, facts, consumed, fixture_dirs)
                }) {
                    return false;
                }
            }
        }
        matched
    }
}

#[cfg(test)]
#[path = "setups/tests.rs"]
mod tests;
