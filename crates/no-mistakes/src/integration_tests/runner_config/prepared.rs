use super::{
    cache::{collect_analyses, with_request_program},
    ParsedRunnerConfigs, PreparedIntegrationRunnerConfigs, ProjectResult, RunnerConfigFileFacts,
    RunnerConfigSpec,
};
use crate::codebase::ts_resolver::TsConfig;
use crate::config::v2::schema::{NoMistakesConfig, StringOrList, TestProjectPolicy};
use crate::integration_tests::project_config::{self, ConfigProjectInput};
use crate::integration_tests::{analysis, types::Framework};
use anyhow::Result;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

pub(crate) fn prepare(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
    tsconfig: &TsConfig,
) -> PreparedIntegrationRunnerConfigs {
    let mut specs = Vec::new();
    add_framework_specs(
        &mut specs,
        root,
        Framework::Playwright,
        config.tests.playwright.configs.as_ref(),
        &config.tests.playwright.projects,
        visible_paths,
    );
    add_framework_specs(
        &mut specs,
        root,
        Framework::Vitest,
        config.tests.vitest.configs.as_ref(),
        &config.tests.vitest.projects,
        visible_paths,
    );
    PreparedIntegrationRunnerConfigs {
        root: root.to_path_buf(),
        specs,
        tsconfig: tsconfig.clone(),
        visible_files: visible_paths
            .iter()
            .map(|path| crate::codebase::ts_resolver::normalize_path(path))
            .collect(),
    }
}

fn add_framework_specs(
    specs: &mut Vec<RunnerConfigSpec>,
    root: &Path,
    framework: Framework,
    configs: Option<&StringOrList>,
    policies: &BTreeMap<String, TestProjectPolicy>,
    visible_paths: &[PathBuf],
) {
    let needs_projects = policies
        .values()
        .any(|policy| !policy.integration_suites.is_empty() && policy.include.is_empty());
    if !needs_projects {
        return;
    }
    let raw_configs = configs.map_or_else(
        || project_config::discovered_config_paths(root, framework, visible_paths),
        StringOrList::values,
    );
    specs.extend(raw_configs.into_iter().map(|raw| RunnerConfigSpec {
        framework,
        path: crate::codebase::ts_resolver::normalize_path(&root.join(&raw)),
        raw,
    }));
}

impl PreparedIntegrationRunnerConfigs {
    pub(crate) fn paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.specs.iter().map(|spec| &spec.path)
    }

    pub(crate) fn contains(&self, path: &Path) -> bool {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        self.specs.iter().any(|spec| spec.path == path)
    }

    pub(crate) fn parse_program(
        &self,
        path: &Path,
        program: &oxc_ast::ast::Program<'_>,
        source: &str,
    ) -> Option<RunnerConfigFileFacts> {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        let specs = self
            .specs
            .iter()
            .filter(|spec| spec.path == path)
            .collect::<Vec<_>>();
        if specs.is_empty() {
            return None;
        }
        let config_dir = path.parent().unwrap_or(&self.root);
        let (results, mut analyses) = collect_analyses(|| {
            specs
                .into_iter()
                .map(|spec| ProjectResult {
                    framework: spec.framework,
                    raw: spec.raw.clone(),
                    projects: project_config::load_config_projects_from_program(
                        ConfigProjectInput {
                            root: &self.root,
                            framework: spec.framework,
                            raw: &spec.raw,
                            path: &path,
                            source,
                            config_dir,
                            tsconfig: &self.tsconfig,
                        },
                        program,
                        Some(&self.visible_files),
                    )
                    .map_err(|error| error.to_string()),
                })
                .collect()
        });
        analyses.insert(
            path.clone(),
            analysis::analyze_program(&path, program, source),
        );
        Some(RunnerConfigFileFacts { results, analyses })
    }

    pub(crate) fn parse_error(
        &self,
        path: &Path,
        message: String,
    ) -> Option<RunnerConfigFileFacts> {
        let path = crate::codebase::ts_resolver::normalize_path(path);
        let results = self
            .specs
            .iter()
            .filter(|spec| spec.path == path)
            .map(|spec| ProjectResult {
                framework: spec.framework,
                raw: spec.raw.clone(),
                projects: Err(message.clone()),
            })
            .collect::<Vec<_>>();
        (!results.is_empty()).then_some(RunnerConfigFileFacts {
            results,
            analyses: BTreeMap::new(),
        })
    }

    pub(crate) fn parse_all(&self) -> Result<ParsedRunnerConfigs> {
        self.with_request_cache(None, || self.parse_all_inner()).0
    }

    fn parse_all_inner(&self) -> Result<ParsedRunnerConfigs> {
        let mut parsed = ParsedRunnerConfigs::default();
        let mut seen = HashSet::new();
        for spec in &self.specs {
            if !seen.insert(spec.path.clone()) {
                continue;
            }
            if !spec.path.exists() {
                anyhow::bail!(
                    "{} config does not exist: {}",
                    spec.framework.as_str(),
                    spec.path.display()
                );
            }
            let source = std::fs::read_to_string(&spec.path)?;
            let facts = with_request_program(&spec.path, &source, |program, source| {
                self.parse_program(&spec.path, program, source)
                    .expect("runner config path was prepared")
            })?;
            parsed.analyses.extend(facts.analyses.clone());
            parsed.files.insert(spec.path.clone(), facts);
        }
        Ok(parsed)
    }

    pub(crate) fn parse_path_for_facts(&self, path: &Path) -> Option<RunnerConfigFileFacts> {
        if !self.contains(path) || !path.exists() {
            return None;
        }
        let source = match std::fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => return self.parse_error(path, error.to_string()),
        };
        match with_request_program(path, &source, |program, source| {
            self.parse_program(path, program, source)
                .expect("runner config path was prepared")
        }) {
            Ok(facts) => Some(facts),
            Err(error) => self.parse_error(path, error.to_string()),
        }
    }
}
