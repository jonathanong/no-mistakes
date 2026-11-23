use anyhow::{ensure, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use std::{collections::BTreeSet, path::Path};

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub(super) struct RetryOptions {
    pub(super) max_retries: u64,
    pub(super) allowlist: Vec<AllowEntry>,
    pub(super) test_commands: Vec<Vec<String>>,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub(super) struct SerialOptions {
    pub(super) allowlist: Vec<AllowEntry>,
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
