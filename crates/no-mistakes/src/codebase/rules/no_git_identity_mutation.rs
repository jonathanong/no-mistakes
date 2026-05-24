use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod runner;

use runner::is_managed_runner_only;

pub const RULE_ID: &str = "no-git-identity-mutation";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) exclude_paths: Vec<String>,
    pub(crate) conditionally_allowed_workflows: Vec<String>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let files = discover_files(root, &config.filesystem.skip_directories);
    check_with_files(root, config, &files)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        let target_roots = super::target_roots(root, config, rule);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|p| target_roots.iter().any(|r| p.starts_with(r)))
            .cloned()
            .collect();
        findings.extend(scan(root, &opts, &files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

pub(crate) fn build_exclude_globset(patterns: &[String]) -> GlobSet {
    let mut builder = GlobSetBuilder::new();
    for pat in patterns {
        if let Ok(glob) = Glob::new(pat) {
            builder.add(glob);
        }
    }
    builder.build().unwrap_or_else(|_| GlobSet::empty())
}

pub(crate) fn build_patterns() -> [Regex; 3] {
    [
        // Allow line continuations (\<newline>) between `git config` and `user.`
        Regex::new(
            r#"(?m)(^|[^a-zA-Z0-9_])git[ \t]+config(?:[^\n\\]|\\\n[ \t]*)*[ \t][`"']?user\.(name|email)[`"']?(?:[ \t]|\\\n[ \t]*)+[^#\s\\|&;<>]"#,
        )
        .expect("shell pattern"),
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

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let exclude_set = build_exclude_globset(&opts.exclude_paths);
    let cond_set = build_exclude_globset(&opts.conditionally_allowed_workflows);
    let patterns = build_patterns();

    let findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file(path, root, &exclude_set, &cond_set, &patterns))
        .collect();
    Ok(findings)
}

pub(crate) fn check_file(
    path: &Path,
    root: &Path,
    exclude_set: &GlobSet,
    cond_set: &GlobSet,
    patterns: &[Regex; 3],
) -> Vec<RuleFinding> {
    let rel_str = relative_slash_path(root, path);

    if exclude_set.is_match(rel_str.as_str()) {
        return Vec::new();
    }

    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !matches!(ext, "sh" | "bash" | "zsh" | "fish" | "yml" | "yaml")
        && !has_shell_shebang(&content)
    {
        return Vec::new();
    }

    if cond_set.is_match(rel_str.as_str()) && is_managed_runner_only(&content) {
        return Vec::new();
    }

    let file = relative_slash_path(root, path);
    let mut findings = Vec::new();

    for pattern_re in patterns {
        for mat in pattern_re.find_iter(&content) {
            if is_read_only_config_match(mat.as_str()) {
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
mod tests;
