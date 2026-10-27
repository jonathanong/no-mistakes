use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
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

super::sourced_file_rule_entrypoints!();

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

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Result<Vec<RuleFinding>> {
    let exclude_set = build_exclude_globset(&opts.exclude_paths);
    let cond_set = build_exclude_globset(&opts.conditionally_allowed_workflows);
    let patterns = build_patterns();

    let findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| {
            check_file_with_sources(path, root, &exclude_set, &cond_set, &patterns, sources)
        })
        .collect();
    Ok(findings)
}

fn check_file_with_sources(
    path: &Path,
    root: &Path,
    exclude_set: &GlobSet,
    cond_set: &GlobSet,
    patterns: &[Regex; 3],
    sources: &crate::codebase::ts_source::SourceStore,
) -> Vec<RuleFinding> {
    let Some(content) = super::read_source(sources, path) else {
        return Vec::new();
    };
    check_source(path, root, exclude_set, cond_set, patterns, &content)
}

fn check_source(
    path: &Path,
    root: &Path,
    exclude_set: &GlobSet,
    cond_set: &GlobSet,
    patterns: &[Regex; 3],
    content: &str,
) -> Vec<RuleFinding> {
    let rel_str = relative_slash_path(root, path);
    if exclude_set.is_match(rel_str.as_str()) {
        return Vec::new();
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !matches!(ext, "sh" | "bash" | "zsh" | "fish" | "yml" | "yaml")
        && !has_shell_shebang(content)
    {
        return Vec::new();
    }

    if cond_set.is_match(rel_str.as_str()) && is_managed_runner_only(content) {
        return Vec::new();
    }

    let file = relative_slash_path(root, path);
    let mut findings = Vec::new();

    for pattern_re in patterns {
        for captures in pattern_re.captures_iter(content) {
            let mat = captures.get(0).expect("complete match");
            // A global option argument named --get is not a config query flag.
            let config_arguments = captures
                .name("config")
                .map_or(mat.as_str(), |config| &content[config.end()..mat.end()]);
            if is_read_only_config_match(config_arguments) {
                continue;
            }
            if mat
                .as_str()
                .chars()
                .last()
                .is_some_and(|c| c.is_ascii_digit())
                && content[mat.end()..].starts_with('>')
            {
                continue;
            }
            let prefix = &content[..mat.start()];
            // When the match starts at '\n', the actual keyword is on the following
            // line; adjust line_start accordingly before checking for comments.
            let starts_at_newline = content.as_bytes().get(mat.start()) == Some(&b'\n');
            let line_start = if starts_at_newline {
                mat.start() + 1
            } else {
                prefix.rfind('\n').map_or(0, |i| i + 1)
            };
            let line_text = content[line_start..]
                .lines()
                .next()
                .unwrap_or("")
                .trim_start();
            if line_text.starts_with('#')
                || line_text.starts_with("echo ")
                || line_text.starts_with("printf ")
            {
                continue;
            }
            let newline_count = prefix.bytes().filter(|&b| b == b'\n').count();
            let line = newline_count + 1 + usize::from(starts_at_newline);
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: file.clone(),
                line,
                message: format!(
                    "{file}:{line}: git config user.name/email is banned \
                    \u{2014} use GIT_AUTHOR_*/GIT_COMMITTER_* env vars instead"
                ),
                import: None,
                target: None,
            });
        }
    }

    findings
}

fn is_read_only_config_match(matched: &str) -> bool {
    matched
        .split(|c: char| c.is_whitespace() || matches!(c, ',' | '(' | ')' | '[' | ']'))
        .any(|token| {
            let token = token.trim_matches(|c| matches!(c, '\'' | '"' | '`'));
            matches!(
                token,
                "--get" | "--get-all" | "--get-regexp" | "--get-urlmatch"
            )
        })
}

#[cfg(test)]
mod prefix_tests;
#[cfg(test)]
mod runner_tests;
#[cfg(test)]
mod tests;
