use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "package-json-registry-only";

mod lockfile;

const BLOCKED_PREFIXES: &[&str] = &[
    "git:",
    "git+ssh:",
    "git+https:",
    "github:",
    "bitbucket:",
    "gitlab:",
    "gist:",
    "http:",
    "https:",
    "file:",
    "link:",
    "portal:",
    "patch:",
];

#[rustfmt::skip]
const DEP_FIELDS: &[&str] = &["dependencies", "devDependencies", "peerDependencies", "optionalDependencies"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) lockfile: Option<PathBuf>,
    pub(crate) scopes: Vec<PathBuf>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let files = discover_files(root, &config.filesystem.skip_directories);
    check_with_files(root, config, &files)
}

super::borrowed_source_rule_entrypoints!();
mod effort;
pub(crate) use effort::check_with_files_sources_and_deferred_suppression;
pub use effort::diagnostics_with_files_and_sources;

pub(crate) fn is_blocked_specifier(spec: &str) -> bool {
    if spec.starts_with("workspace:") || spec.starts_with("catalog:") {
        return false;
    }
    if let Some(rest) = spec.strip_prefix("npm:") {
        if let Some(after_at) = rest.strip_prefix('@') {
            let version = after_at.find('@').map_or("", |i| &after_at[i + 1..]);
            return !version.is_empty() && is_blocked_specifier(version);
        }
        return is_blocked_specifier(rest.rfind('@').map_or(rest, |i| &rest[i + 1..]));
    }
    BLOCKED_PREFIXES.iter().any(|p| spec.starts_with(p))
        || (!spec.starts_with('@') && spec.contains('/'))
        || (spec.starts_with('@') && spec.splitn(3, '/').count() > 2)
}

mod scan;

#[cfg(test)]
mod tests;
