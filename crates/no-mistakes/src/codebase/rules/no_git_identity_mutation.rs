use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "no-git-identity-mutation";

/// Runners that are considered non-self-hosted (GitHub-managed).
const MANAGED_RUNNERS: &[&str] = &[
    "ubuntu-latest",
    "ubuntu-22.04",
    "ubuntu-24.04",
    "ubuntu-slim",
    "ubuntu-22.04-slim",
    "macos-latest",
    "windows-latest",
];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) exclude_paths: Vec<String>,
    pub(crate) conditionally_allowed_workflows: Vec<String>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let skip = &config.filesystem.skip_directories;
    let files = discover_files(root, skip);
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        findings.extend(scan(root, &opts, &files)?);
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

/// Check using a pre-discovered file list.
pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(RULE_ID) {
        let opts: Options = rule.rule_options();
        findings.extend(scan(root, &opts, all_files)?);
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
    // Three patterns ported from filaments no-git-identity-mutation.
    // QUOTES = [`"'] — any quote character (backtick, double-quote, single-quote)
    [
        // Shell form: git config [opts] user.name|email
        Regex::new(
            r#"(?m)(^|[^a-zA-Z0-9_])git[ \t]+config[^\n]*[ \t][`"']?user\.(name|email)([^a-zA-Z0-9.-]|$)"#,
        )
        .expect("shell pattern"),
        // Array form: ['git', 'config', ..., 'user.name|email']
        Regex::new(
            r#"(?ms)[`"']git[`"'].{0,500}?[`"']config[`"']\s*,\s*(?:[`"']--[a-zA-Z0-9-]+[`"']\s*,\s*)*[`"']user\.(name|email)[`"']"#,
        )
        .expect("array pattern"),
        // Helper form: git('config', ..., 'user.name|email')
        Regex::new(
            r#"(?m)(^|[^a-zA-Z0-9_])git\s*\(\s*[`"']config[`"']\s*,\s*(?:[`"']--[a-zA-Z0-9-]+[`"']\s*,\s*)*[`"']user\.(name|email)[`"']"#,
        )
        .expect("helper pattern"),
    ]
}

pub(crate) fn is_managed_runner_only(content: &str) -> bool {
    // Find all runs-on: values; if any is not in MANAGED_RUNNERS, not safe to skip
    let runs_on_re = Regex::new(r"runs-on:\s*(\S+)").expect("runs-on regex");
    let values: Vec<&str> = runs_on_re
        .captures_iter(content)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str()))
        .collect();
    if values.is_empty() {
        // No runs-on at all — not a workflow, don't skip
        return false;
    }
    values.iter().all(|v| MANAGED_RUNNERS.contains(v))
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
    let rel = path.strip_prefix(root).unwrap_or(path);
    let rel_str = rel.to_string_lossy();

    if exclude_set.is_match(rel_str.as_ref()) {
        return Vec::new();
    }

    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };

    // If file matches conditionally_allowed_workflows and uses only managed runners, skip
    if cond_set.is_match(rel_str.as_ref()) && is_managed_runner_only(&content) {
        return Vec::new();
    }

    let file = relative_slash_path(root, path);
    let mut findings = Vec::new();

    for pattern_re in patterns {
        for mat in pattern_re.find_iter(&content) {
            let prefix = &content[..mat.start()];
            let newline_count = prefix.bytes().filter(|&b| b == b'\n').count();
            // The pattern starts with (^|[^a-zA-Z0-9_]) — if the match starts at
            // a newline character (the preceding-char branch), the actual keyword
            // is on the following line, so bump the count by 1.
            let starts_at_newline = content.as_bytes().get(mat.start()) == Some(&b'\n');
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

#[cfg(test)]
mod tests;
