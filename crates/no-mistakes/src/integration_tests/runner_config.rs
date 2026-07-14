use super::types::{ConfigProject, FileAnalysis, Framework};
use crate::codebase::ts_resolver::TsConfig;
use anyhow::Result;
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

mod cache;
mod prepared;
#[cfg(test)]
mod tests;
pub(in crate::integration_tests) use cache::{read_request_source, with_program};
pub(crate) use prepared::{prepare, prepare_with_sources};

#[derive(Clone)]
pub(crate) struct RunnerConfigFactPlan {
    pub(crate) root: PathBuf,
    pub(crate) primary_files: HashSet<PathBuf>,
    pub(crate) graph_files: HashSet<PathBuf>,
    pub(crate) primary_plan: crate::codebase::check_facts::CheckFactPlan,
    pub(crate) graph_plan: crate::codebase::check_facts::CheckFactPlan,
    pub(crate) playwright: Option<crate::codebase::check_facts::PlaywrightFactPlan>,
}

#[derive(Clone)]
struct RunnerConfigSpec {
    framework: Framework,
    raw: String,
    path: PathBuf,
}

#[derive(Clone)]
pub struct PreparedIntegrationRunnerConfigs {
    root: PathBuf,
    specs: Vec<RunnerConfigSpec>,
    tsconfig: TsConfig,
    visible_files: HashSet<PathBuf>,
    sources: Option<std::sync::Arc<crate::codebase::ts_source::SourceStore>>,
}

#[derive(Clone)]
struct ProjectResult {
    framework: Framework,
    raw: String,
    projects: Result<Vec<ConfigProject>, String>,
}

#[derive(Clone, Default)]
pub(crate) struct RunnerConfigFileFacts {
    results: Vec<ProjectResult>,
    analyses: BTreeMap<PathBuf, FileAnalysis>,
}

#[derive(Clone, Default)]
pub(crate) struct ParsedRunnerConfigs {
    files: BTreeMap<PathBuf, RunnerConfigFileFacts>,
    analyses: BTreeMap<PathBuf, FileAnalysis>,
}

impl ParsedRunnerConfigs {
    pub(crate) fn with_files(files: BTreeMap<PathBuf, RunnerConfigFileFacts>) -> Self {
        let analyses = files
            .values()
            .flat_map(|facts| facts.analyses.iter())
            .map(|(path, analysis)| (path.clone(), analysis.clone()))
            .collect();
        Self { files, analyses }
    }

    pub(crate) fn covers(&self, plan: &PreparedIntegrationRunnerConfigs) -> bool {
        plan.specs.iter().all(|spec| {
            !spec.path.exists()
                || self.files.get(&spec.path).is_some_and(|facts| {
                    facts
                        .results
                        .iter()
                        .any(|result| result.framework == spec.framework && result.raw == spec.raw)
                })
        })
    }

    pub(crate) fn analyses_for(&self, source_files: &[PathBuf]) -> BTreeMap<PathBuf, FileAnalysis> {
        let source_files = source_files.iter().collect::<HashSet<_>>();
        self.analyses
            .iter()
            .filter(|(path, _)| source_files.contains(path))
            .map(|(path, analysis)| (path.clone(), analysis.clone()))
            .collect()
    }

    pub(crate) fn projects_for(
        &self,
        plan: &PreparedIntegrationRunnerConfigs,
        framework: Framework,
    ) -> Result<Vec<ConfigProject>> {
        let mut projects = Vec::new();
        for spec in plan.specs.iter().filter(|spec| spec.framework == framework) {
            if !spec.path.exists() {
                anyhow::bail!(
                    "{} config does not exist: {}",
                    framework.as_str(),
                    spec.path.display()
                );
            }
            let file = self.files.get(&spec.path).ok_or_else(|| {
                anyhow::anyhow!(
                    "missing prepared {} config: {}",
                    framework.as_str(),
                    spec.raw
                )
            })?;
            let result = file
                .results
                .iter()
                .find(|result| result.framework == framework && result.raw == spec.raw)
                .expect("prepared runner config result must match its spec");
            projects.extend(result.projects.clone().map_err(anyhow::Error::msg)?);
        }
        Ok(projects)
    }
}
