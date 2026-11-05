use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};

use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod prefix;
mod runner;
use prefix::shell_pattern;

use runner::is_managed_runner_only;

pub const RULE_ID: &str = "no-git-identity-mutation";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) exclude_paths: Vec<String>,
    pub(crate) conditionally_allowed_workflows: Vec<String>,
}

mod audit;
mod scan;
pub(crate) use audit::check_with_files_and_sources;
#[cfg(test)]
use scan::check_file_with_sources;
use scan::check_source;

pub fn check(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
) -> Result<Vec<RuleFinding>> {
    super::check_sourced_standalone(root, config, check_with_files)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &crate::config::v2::NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    super::check_with_borrowed_source_store(root, config, files, check_with_files_and_sources)
}

pub(crate) fn build_exclude_globset(patterns: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for pat in patterns {
        if let Ok(glob) = Glob::new(pat) {
            builder.add(glob);
        }
    }
    builder.build().expect("compiled globset")
}

pub(crate) fn build_patterns() -> [Regex; 3] {
    [
        // Allow line continuations (\<newline>) between `git config` and `user.`
        shell_pattern(),
        // Array form: ['git', 'config', ..., 'user.name|email', 'value']
        Regex::new(
            r#"(?ms)[`"']git[`"'].{0,500}?[`"']config[`"']\s*,\s*(?:[`"']--[a-zA-Z0-9-]+[`"']\s*,\s*)*[`"']user\.(name|email)[`"']\s*,\s*[^,\]\)]"#,
        )
        .expect("array pattern"),
        // Helper form: git('config', ..., 'user.name|email', 'value')
        Regex::new(
            r#"(?m)(^|[^a-zA-Z0-9_])git\s*\(\s*[`"']config[`"']\s*,\s*(?:[`"']--[a-zA-Z0-9-]+[`"']\s*,\s*)*[`"']user\.(name|email)[`"']\s*,\s*[^,\]\)]"#,
        )
        .expect("helper pattern"),
    ]
}

pub(crate) fn has_shell_shebang(content: &str) -> bool {
    let l = content.lines().next().unwrap_or("");
    l.starts_with("#!/bin/sh")
        || l.starts_with("#!/bin/bash")
        || l.starts_with("#!/usr/bin/env sh")
        || l.starts_with("#!/usr/bin/env bash")
}

#[cfg(test)]
mod prefix_tests;
#[cfg(test)]
mod runner_tests;
#[cfg(test)]
mod tests;
