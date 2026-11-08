//! Lexical configured roots for directory rules.
//!
//! `root.join("services/../agents")` still contains `..`, so `strip_prefix`
//! misses files under `agents`. Normalize before any prefix check. A result
//! that is not under the repository is not scanned.

use super::RuleFinding;
use std::path::{Path, PathBuf};

pub(crate) fn normalize_configured_root(repo: &Path, configured: &Path) -> PathBuf {
    let joined = if configured.is_absolute() {
        configured.to_path_buf()
    } else {
        repo.join(configured)
    };
    crate::codebase::ts_resolver::normalize_path(&joined)
}

pub(crate) fn escapes_repository(repo: &Path, candidate: &Path) -> bool {
    let repo = crate::codebase::ts_resolver::normalize_path(repo);
    let candidate = crate::codebase::ts_resolver::normalize_path(candidate);
    // `normalize_path(".")` is empty, and every path starts with an empty path.
    if repo.as_os_str().is_empty() {
        return candidate.is_absolute()
            || candidate.components().next() == Some(std::path::Component::ParentDir);
    }
    !candidate.starts_with(&repo)
}

pub(crate) fn escape_finding(rule: &str, configured: &Path, config_file: &str) -> RuleFinding {
    let display = configured.to_string_lossy().replace('\\', "/");
    RuleFinding {
        causes: Vec::new(),
        rule: rule.to_string(),
        file: config_file.to_string(),
        line: 1,
        message: format!("{display}: configured root escapes the repository"),
        import: None,
        target: None,
    }
}

/// The config file the loader read, so `no-mistakes-disable-file` applies to it.
///
/// `check` canonicalizes the repository root. On Windows that adds a `\\?\`
/// prefix, so a lexical config path would otherwise stay absolute.
pub(crate) fn config_file_for_finding(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
) -> String {
    let Some(path) = config.loaded_config_path.0.as_deref() else {
        return ".no-mistakes.yml".to_string();
    };
    repository_relative(root, path)
}

pub(crate) fn config_file_for_rule(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
) -> String {
    let origin = config
        .rules
        .iter()
        .position(|entry| std::ptr::eq(entry, rule))
        .and_then(|index| {
            config.config_provenance.origin(
                &crate::config::source_locations::ConfigPath::default()
                    .key("rules")
                    .index(index),
            )
        });
    origin
        .map(|(path, _)| repository_relative(root, path))
        .unwrap_or_else(|| config_file_for_finding(root, config))
}

fn repository_relative(root: &Path, path: &Path) -> String {
    if let Some(relative) = slash_relative(root, path) {
        return relative;
    }
    let root_text = without_verbatim_prefix(&slash(root));
    let path_text = without_verbatim_prefix(&slash(path));
    slash_relative(Path::new(&root_text), Path::new(&path_text)).unwrap_or_else(|| slash(path))
}

fn slash_relative(root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root).ok().map(slash)
}

fn slash(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn without_verbatim_prefix(path: &str) -> String {
    let Some(rest) = path.strip_prefix("//?/") else {
        return path.to_string();
    };
    rest.strip_prefix("UNC/").unwrap_or(rest).to_string()
}

pub(crate) fn run_with_origin<O: serde::de::DeserializeOwned + Default>(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
    rule_id: &str,
    enabled: impl Fn(&O) -> bool + Sync,
    scan: impl Fn(&Path, &O, &[PathBuf], &str) -> anyhow::Result<Vec<RuleFinding>> + Sync,
) -> anyhow::Result<Vec<RuleFinding>> {
    use rayon::prelude::*;
    let groups: anyhow::Result<Vec<_>> = config
        .rule_applications(rule_id)
        .into_par_iter()
        .map(|rule| {
            let options = rule.try_rule_options()?;
            if !enabled(&options) {
                return Ok(Vec::new());
            }
            let selected = super::selected_files::selected_rule_files(root, config, rule, files)?;
            scan(
                root,
                &options,
                &selected,
                &config_file_for_rule(root, config, rule),
            )
        })
        .collect();
    let mut findings = groups?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

#[cfg(test)]
mod tests;
