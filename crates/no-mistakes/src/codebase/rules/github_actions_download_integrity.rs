use super::RuleFinding;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod assign;
mod effort;
mod env;
mod judge;
mod runs;
mod url;
mod verify;
mod words;

pub const RULE_ID: &str = "github-actions-download-integrity";

pub(super) const DEFAULT_TRUSTED_HOSTS: &[&str] = &[
    "localhost",
    "127.0.0.1",
    "api.github.com",
    "objects.githubusercontent.com",
];

/// `trustedHosts` replaces the default list when present, including `[]`.
/// `downloadHelpers` adds command names beside `curl` and `wget`.
#[derive(Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
struct Options {
    trusted_hosts: Option<Vec<String>>,
    download_helpers: Vec<String>,
    shell_source_include: Vec<String>,
    strict_sequencing: bool,
}

fn compile_hosts(opts: &Options) -> Result<Vec<String>> {
    let hosts = match &opts.trusted_hosts {
        None => DEFAULT_TRUSTED_HOSTS
            .iter()
            .map(|host| (*host).to_string())
            .collect(),
        Some(list) => list
            .iter()
            .map(|host| {
                judge::configured_host(host).ok_or_else(|| {
                    anyhow::anyhow!("{RULE_ID} trustedHosts entry `{host}` is not a host")
                })
            })
            .collect::<Result<_>>()?,
    };
    for helper in &opts.download_helpers {
        if !valid_helper(helper) {
            anyhow::bail!("{RULE_ID} downloadHelpers entry `{helper}` is not a command name");
        }
    }
    Ok(hosts)
}

fn valid_helper(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

pub use effort::{check, diagnostics_with_files_and_sources};
pub(crate) use effort::{check_with_files, check_with_snapshot};

/// `Some(true)` is an action file. `Some(false)` is a workflow. `None` is ignored.
pub(super) fn gha_kind(rel: &str) -> Option<bool> {
    let yaml = rel.ends_with(".yml") || rel.ends_with(".yaml");
    if !yaml {
        return None;
    }
    // GitHub only runs a workflow file directly under `.github/workflows/`.
    if let Some(name) = rel.strip_prefix(".github/workflows/") {
        if !name.is_empty() && !name.contains('/') {
            return Some(false);
        }
        return None;
    }
    if rel.starts_with(".github/actions/")
        && (rel.ends_with("/action.yml") || rel.ends_with("/action.yaml"))
    {
        return Some(true);
    }
    None
}

#[cfg(test)]
mod coverage;
#[cfg(test)]
mod edges;
#[cfg(test)]
mod effort_tests;
#[cfg(test)]
mod test_adapters;
#[cfg(test)]
mod tests;
