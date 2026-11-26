use crate::codebase::rules::path_filter::GlobMatcher;
use crate::fx::FxHashSet;
use anyhow::{ensure, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use std::{collections::BTreeSet, path::Path};

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub(in crate::codebase::rules) struct RetryOptions {
    pub(super) max_retries: u64,
    pub(super) allowlist: Vec<AllowEntry>,
    pub(super) test_commands: Vec<Vec<String>>,
    pub(super) files: Vec<String>,
    pub(super) fixtures: Vec<FixtureExportEntry>,
}

impl RetryOptions {
    pub(super) fn factory_files(&self) -> Result<GlobMatcher> {
        ensure!(
            self.files.iter().all(|glob| !glob.trim().is_empty()),
            "test-no-retries: options.files entries must be nonempty globs"
        );
        GlobMatcher::new(&self.files, "test-no-retries options.files")
    }
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub(in crate::codebase::rules) struct SerialOptions {
    pub(super) allowlist: Vec<AllowEntry>,
    pub(super) fixtures: Vec<FixtureExportEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FixtureExportEntry {
    module: String,
    export: String,
}

pub(super) fn compile_fixtures(
    id: &str,
    entries: Vec<FixtureExportEntry>,
) -> Result<FxHashSet<(String, String)>> {
    let mut fixtures = FxHashSet::default();
    for (index, entry) in entries.into_iter().enumerate() {
        let field = format!("{id}: fixtures[{index}]");
        let module = entry.module.trim().trim_start_matches("./");
        ensure!(
            !module.is_empty()
                && !Path::new(module).is_absolute()
                && !module.contains('\\')
                && !module.contains(':')
                && !module.split('/').any(|component| component == ".."),
            "{field}: module must be a non-empty repository-relative path"
        );
        let export = entry.export.trim();
        ensure!(!export.is_empty(), "{field}: export must not be blank");
        fixtures.insert((module.to_string(), export.to_string()));
    }
    Ok(fixtures)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AllowEntry {
    path: Option<String>,
    project: Option<String>,
    reason: String,
}

pub(super) struct Allowlist {
    pub(super) paths: GlobSet,
    pub(super) projects: BTreeSet<String>,
}

impl Allowlist {
    pub(super) fn compile(id: &str, entries: Vec<AllowEntry>) -> Result<Self> {
        let mut paths = GlobSetBuilder::new();
        let mut projects = BTreeSet::new();
        for (index, entry) in entries.into_iter().enumerate() {
            let field = format!("{id}: allowlist[{index}]");
            ensure!(
                !entry.reason.trim().is_empty(),
                "{field}: reason must not be blank"
            );
            match (entry.path, entry.project) {
                (Some(path), None) => {
                    let path = path.trim().trim_start_matches("./");
                    ensure!(
                        !path.is_empty()
                            && !Path::new(path).is_absolute()
                            && !path.contains('\\')
                            && !path.contains(':')
                            && !path.split('/').any(|component| component == ".."),
                        "{field}: path must be a non-empty repository-relative glob"
                    );
                    paths
                        .add(Glob::new(path).map_err(|error| anyhow::anyhow!("{field}: {error}"))?);
                }
                (None, Some(project)) if !project.trim().is_empty() => {
                    projects.insert(project.trim().to_string());
                }
                _ => anyhow::bail!("{field}: provide exactly one non-empty path or project"),
            }
        }
        Ok(Self {
            paths: paths.build()?,
            projects,
        })
    }
}

pub(super) fn validate_test_commands(commands: &[Vec<String>]) -> Result<()> {
    for (index, command) in commands.iter().enumerate() {
        ensure!(
            !command.is_empty() && command.iter().all(|word| !word.trim().is_empty()),
            "test-no-retries: testCommands[{index}] must be a non-empty literal argv prefix"
        );
    }
    Ok(())
}

pub(in crate::codebase::rules) fn validate_test_no_retries(options: RetryOptions) -> Result<()> {
    validate_test_commands(&options.test_commands)?;
    options.factory_files()?;
    Allowlist::compile(super::TEST_NO_RETRIES, options.allowlist)?;
    compile_fixtures(super::TEST_NO_RETRIES, options.fixtures)?;
    Ok(())
}

pub(in crate::codebase::rules) fn validate_playwright_no_serial(
    options: SerialOptions,
) -> Result<()> {
    Allowlist::compile(super::PLAYWRIGHT_NO_SERIAL, options.allowlist)?;
    compile_fixtures(super::PLAYWRIGHT_NO_SERIAL, options.fixtures)?;
    Ok(())
}
