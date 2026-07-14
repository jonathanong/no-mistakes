use super::types::{ConfigProject, FileAnalysis, Suite};
use crate::codebase::ts_resolver::TsConfig;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

pub(super) fn tsconfig_without_config(root: &Path) -> TsConfig {
    TsConfig {
        dir: root.to_path_buf(),
        paths: Vec::new(),
        paths_dir: root.to_path_buf(),
        base_url: None,
    }
}

pub(super) fn configured_suites(root: &Path, config: &NoMistakesConfig) -> Result<Vec<Suite>> {
    let snapshot = crate::codebase::ts_source::VisiblePathSnapshot::new(root);
    let visible_paths = snapshot.paths_for(root);
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, root, &visible_paths)?;
    let runner_configs = super::runner_config::prepare(root, config, &visible_paths, &tsconfig);
    let parsed = runner_configs.parse_all()?;
    super::config::configured_suites_from_runner_configs(root, config, &runner_configs, &parsed)
}

pub(super) fn analyze_files(files: &[PathBuf]) -> Result<BTreeMap<PathBuf, FileAnalysis>> {
    super::analysis::analyze_files_with_seed(files, BTreeMap::new())
}

pub(super) fn parse_playwright(
    source: &str,
    path: &Path,
    config_dir: &Path,
    tsconfig: &TsConfig,
) -> Result<super::test_config::playwright::ParsedPlaywrightConfig> {
    crate::integration_tests::runner_config::with_program(path, source, |program, source| {
        super::test_config::playwright::parse_program(
            program, source, path, config_dir, tsconfig, None,
        )
    })?
}

pub(super) fn parse_playwright_from_visible(
    source: &str,
    path: &Path,
    config_dir: &Path,
    tsconfig: &TsConfig,
    visible_files: &HashSet<PathBuf>,
) -> Result<super::test_config::playwright::ParsedPlaywrightConfig> {
    crate::integration_tests::runner_config::with_program(path, source, |program, source| {
        super::test_config::playwright::parse_program(
            program,
            source,
            path,
            config_dir,
            tsconfig,
            Some(visible_files),
        )
    })?
}

pub(super) fn parse_vitest(
    source: &str,
    path: &Path,
    config_dir: &Path,
    root: &Path,
    tsconfig: &TsConfig,
) -> Result<Vec<ConfigProject>> {
    crate::integration_tests::runner_config::with_program(path, source, |program, source| {
        super::test_config::vitest::parse_program(
            program, source, path, config_dir, root, tsconfig, None,
        )
    })?
}

pub(super) fn parse_vitest_from_visible(
    source: &str,
    path: &Path,
    config_dir: &Path,
    root: &Path,
    tsconfig: &TsConfig,
    visible_files: &HashSet<PathBuf>,
) -> Result<Vec<ConfigProject>> {
    crate::integration_tests::runner_config::with_program(path, source, |program, source| {
        super::test_config::vitest::parse_program(
            program,
            source,
            path,
            config_dir,
            root,
            tsconfig,
            Some(visible_files),
        )
    })?
}
