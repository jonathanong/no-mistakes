use super::RuleFinding;
use crate::codebase::ts_source::{has_disable_file_comment, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "rust-max-lines-per-file";

pub(crate) const DEFAULT_SRC_MAX: usize = 200;
pub(crate) const DEFAULT_TEST_MAX: usize = 500;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) src_max: Option<usize>,
    pub(crate) test_max: Option<usize>,
    pub(crate) excludes: Vec<String>,
    pub(crate) roots: Option<Vec<PathBuf>>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    super::rust_rules_combined::standalone::check(root, config, RULE_ID)
}

pub(crate) fn check_source_with_deferred_suppression(
    path: &Path,
    root: &Path,
    content: &str,
    limit: usize,
    defer_suppression: bool,
) -> Option<RuleFinding> {
    if !defer_suppression && has_disable_file_comment(content, RULE_ID) {
        return None;
    }
    let code_lines = count_code_lines(content);
    if code_lines <= limit {
        return None;
    }
    Some(RuleFinding {
        rule: RULE_ID.to_string(),
        file: relative_slash_path(root, path),
        line: 1,
        message: format!("{code_lines} code lines (max {limit}) - split into smaller modules"),
        import: None,
        target: None,
    })
}

pub(crate) fn is_test_file(root: &Path, path: &Path) -> bool {
    let rel = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    rel.contains("/tests/")
        || rel.starts_with("tests/")
        || path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n == "tests.rs")
}

pub(crate) use line_counter::count_code_lines;

mod line_counter;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_support;
