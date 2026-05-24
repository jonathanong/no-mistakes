use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const RULE_ID: &str = "shellcheck-runner";

const DEFAULT_SEVERITY: &str = "warning";
const SHEBANG_BYTES: usize = 256;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct ShellcheckOptions {
    pub(crate) severity: String,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) shell_files: Vec<String>,
    pub(crate) shebang_dirs: Vec<String>,
    pub(crate) shellcheck: ShellcheckOptions,
    pub(crate) skills_lockfile: Option<String>,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let skip = &config.filesystem.skip_directories;
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let files: Vec<PathBuf> = target_roots
                .iter()
                .flat_map(|r| discover_files(r, skip))
                .collect();
            scan(root, &opts, &files)
        })
        .collect();
    merge(all)
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: Options = rule.rule_options();
            let target_roots = super::target_roots(root, config, rule);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| target_roots.iter().any(|r| p.starts_with(r)))
                .cloned()
                .collect();
            scan(root, &opts, &files)
        })
        .collect();
    merge(all)
}

fn merge(all: Result<Vec<Vec<RuleFinding>>>) -> Result<Vec<RuleFinding>> {
    let mut v: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut v);
    Ok(v)
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let shell_candidates = collect_shell_files(root, opts, files);
    if shell_candidates.is_empty() {
        return Ok(Vec::new());
    }
    run_shellcheck(root, opts, &shell_candidates)
}

pub(crate) fn collect_shell_files(root: &Path, opts: &Options, files: &[PathBuf]) -> Vec<PathBuf> {
    let sh = |p: &Path| p.extension().and_then(|e| e.to_str()) == Some("sh");
    let mut candidates: Vec<PathBuf> = files.iter().filter(|p| sh(p)).cloned().collect();
    for dir_rel in &opts.shebang_dirs {
        let dir = if dir_rel.is_empty() {
            root.to_path_buf()
        } else {
            root.join(dir_rel)
        };
        for path in files {
            let Some(parent) = path.parent() else {
                continue;
            };
            if parent != dir || sh(path) {
                continue;
            }
            if has_bash_shebang(path) {
                candidates.push(path.clone());
            }
        }
    }
    for rel in &opts.shell_files {
        let abs = root.join(rel);
        if abs.exists() {
            candidates.push(abs);
        }
    }
    candidates.sort();
    candidates.dedup();
    candidates
}

fn has_bash_shebang(path: &Path) -> bool {
    use std::io::Read;
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut buf = [0u8; SHEBANG_BYTES];
    let n = file.read(&mut buf).unwrap_or(0);
    let header = std::str::from_utf8(&buf[..n]).unwrap_or("");
    let l = header.lines().next().unwrap_or("");
    l.starts_with("#!/bin/bash")
        || l.starts_with("#!/usr/bin/env bash")
        || l.starts_with("#!/bin/sh")
        || l.starts_with("#!/usr/bin/env sh")
}

pub(crate) fn run_shellcheck(
    root: &Path,
    opts: &Options,
    shell_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sev = if opts.shellcheck.severity.is_empty() {
        DEFAULT_SEVERITY
    } else {
        &opts.shellcheck.severity
    };
    let result = Command::new("shellcheck")
        .args(["-f", "gcc", "-S", sev])
        .args(shell_files)
        .output();
    handle_shellcheck_result(root, shell_files, result)
}

pub(crate) fn handle_shellcheck_result(
    root: &Path,
    shell_files: &[PathBuf],
    result: std::io::Result<std::process::Output>,
) -> Result<Vec<RuleFinding>> {
    match result {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(anyhow::anyhow!("failed to run shellcheck: {e}")),
        Ok(output) if output.status.success() => Ok(Vec::new()),
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut findings: Vec<RuleFinding> = parse_affected_files(&stdout, shell_files)
                .iter()
                .map(|path| make_finding(root, path))
                .collect();
            findings.sort_by(|a, b| a.file.cmp(&b.file));
            Ok(findings)
        }
    }
}

fn make_finding(root: &Path, path: &Path) -> RuleFinding {
    let rel = relative_slash_path(root, path);
    let msg = format!("{rel}: shellcheck found issues (run shellcheck manually for details)");
    RuleFinding {
        rule: RULE_ID.to_string(),
        file: rel,
        line: 1,
        message: msg,
        import: None,
        target: None,
    }
}

static GCC_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();

pub(crate) fn parse_affected_files(stdout: &str, shell_files: &[PathBuf]) -> Vec<PathBuf> {
    let re = GCC_RE.get_or_init(|| regex::Regex::new(r"^(.+):\d+:\d+: [a-z]+: ").unwrap());
    let shell_set: std::collections::HashSet<&PathBuf> = shell_files.iter().collect();
    let mut v: Vec<PathBuf> = stdout
        .lines()
        .filter_map(|l| re.captures(l)?.get(1).map(|m| PathBuf::from(m.as_str())))
        .filter(|p| shell_set.contains(p))
        .collect();
    v.sort();
    v.dedup();
    v
}

#[cfg(test)]
mod tests;
