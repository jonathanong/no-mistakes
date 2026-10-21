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
    !candidate.starts_with(&repo)
}

pub(crate) fn escape_finding(rule: &str, configured: &Path) -> RuleFinding {
    let display = configured.to_string_lossy().replace('\\', "/");
    RuleFinding {
        rule: rule.to_string(),
        file: display.clone(),
        line: 1,
        message: format!("{display}: configured root escapes the repository"),
        import: None,
        target: None,
    }
}
