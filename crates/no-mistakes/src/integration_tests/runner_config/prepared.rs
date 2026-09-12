use super::{
    cache::{collect_analyses, with_request_program},
    ParsedRunnerConfigs, PreparedIntegrationRunnerConfigs, ProjectResult, RunnerConfigFileFacts,
    RunnerConfigSpec,
};
use crate::config::v2::schema::{NoMistakesConfig, StringOrList, TestProjectPolicy};
use crate::integration_tests::project_config::{self, ConfigProjectInput};
use crate::integration_tests::{analysis, types::Framework};
use anyhow::Result;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

mod errors;
mod json;
mod setup;

pub(crate) use setup::{prepare, prepare_with_catalog_and_sources};

/// Prepare configured runner files with the request's importer-scoped
/// TypeScript catalog and source store.
#[doc(hidden)]
pub fn prepare_runner_configs_with_catalog(
    root: &Path,
    config: &NoMistakesConfig,
    visible_paths: &[PathBuf],
    tsconfig_catalog: std::sync::Arc<crate::codebase::ts_resolver::TsConfigCatalog>,
    sources: std::sync::Arc<crate::codebase::ts_source::SourceStore>,
) -> PreparedIntegrationRunnerConfigs {
    prepare_with_catalog_and_sources(root, config, visible_paths, tsconfig_catalog, sources)
}

/// Directories containing explicitly configured runner configs. These seed
/// request-local TypeScript config discovery before the runner config itself
/// is parsed, so its imports use the owning package's aliases even outside a
/// declared workspace.
#[doc(hidden)]
pub fn configured_runner_config_dirs(root: &Path, config: &NoMistakesConfig) -> Vec<PathBuf> {
    [
        config.tests.vitest.configs.as_ref(),
        config.tests.playwright.configs.as_ref(),
    ]
    .into_iter()
    .flatten()
    .flat_map(StringOrList::values)
    .filter_map(|path| {
        crate::codebase::ts_resolver::normalize_path(&root.join(path))
            .parent()
            .map(Path::to_path_buf)
    })
    .collect()
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
        let resolver = crate::codebase::ts_resolver::ScopedImportResolver::from_visible(
            &self.tsconfig_catalog,
            &self.visible_files,
        );
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
                            resolver: &resolver,
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

    fn read_source(&self, path: &Path) -> Result<std::sync::Arc<str>> {
        match &self.sources {
            Some(sources) => sources
                .read_path(path)
                .map_err(|error| anyhow::anyhow!("reading {}: {}", path.display(), error)),
            None => super::cache::read_request_source(path),
        }
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
            if !self.visible_files.contains(&spec.path) {
                anyhow::bail!(
                    "{} config does not exist: {}",
                    spec.framework.as_str(),
                    spec.path.display()
                );
            }
            let source = self.read_source(&spec.path).map_err(|error| {
                anyhow::anyhow!(
                    "{} config does not exist or could not be read: {}: {}",
                    spec.framework.as_str(),
                    spec.path.display(),
                    error
                )
            })?;
            let facts = if spec.path.extension().and_then(|value| value.to_str()) == Some("json") {
                self.parse_json(&spec.path, &source)
            } else {
                with_request_program(&spec.path, &source, |program, source| {
                    self.parse_program(&spec.path, program, source)
                        .expect("runner config path was prepared")
                })?
            };
            parsed.analyses.extend(facts.analyses.clone());
            parsed.files.insert(spec.path.clone(), facts);
        }
        Ok(parsed)
    }

    pub(crate) fn parse_path_for_facts_with_session(
        &self,
        session: &crate::codebase::analysis_session::AnalysisSession,
        path: &Path,
    ) -> Option<RunnerConfigFileFacts> {
        if !self.contains(path) || !path.exists() {
            return None;
        }
        let source = match match &self.sources {
            Some(sources) => sources
                .read_path(path)
                .map_err(|error| anyhow::anyhow!("reading {}: {}", path.display(), error)),
            None => super::cache::read_request_source_with_session(session, path),
        } {
            Ok(source) => source,
            Err(error) => return self.parse_error(path, error.to_string()),
        };
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            return Some(self.parse_json(path, &source));
        }
        match session.with_program(path, &source, |program, source| {
            self.parse_program(path, program, source)
                .expect("runner config path was prepared")
        }) {
            Ok(facts) => Some(facts),
            Err(error) => self.parse_error(path, error.to_string()),
        }
    }
}
