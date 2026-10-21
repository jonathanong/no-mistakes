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

/// The config file that declared the root, so `no-mistakes-disable-file` can load it.
pub(crate) fn config_file_for_finding(root: &Path, files: &[PathBuf]) -> String {
    let mut best: Option<String> = None;
    for file in files {
        let rel = crate::codebase::ts_source::relative_slash_path(root, file);
        let name = rel.rsplit('/').next().unwrap_or(rel.as_str());
        if !(name.starts_with(".no-mistakes.") || name == "no-mistakes.yml") {
            continue;
        }
        let replace = match &best {
            Some(current) => rel.len() < current.len(),
            None => true,
        };
        if replace {
            best = Some(rel);
        }
    }
    best.unwrap_or_else(|| ".no-mistakes.yml".to_string())
}

#[cfg(test)]
mod tests;
