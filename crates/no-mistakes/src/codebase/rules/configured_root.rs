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
        rule: rule.to_string(),
        file: config_file.to_string(),
        line: 1,
        message: format!("{display}: configured root escapes the repository"),
        import: None,
        target: None,
    }
}

/// The config file the loader read, so `no-mistakes-disable-file` applies to it.
pub(crate) fn config_file_for_finding(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
) -> String {
    config
        .loaded_config_path
        .relative_to(root)
        .unwrap_or_else(|| ".no-mistakes.yml".to_string())
}

#[cfg(test)]
mod tests;
