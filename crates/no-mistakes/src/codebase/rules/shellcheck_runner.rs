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
const VALID_SEVERITIES: [&str; 4] = ["error", "warning", "info", "style"];

mod candidates;
use candidates::filtered_shell_files;

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
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            let rule_filter = super::path_filter::RulePathFilter::new(root, config, rule)?;
            scan(root, &opts, &files, &target_roots, &rule_filter)
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
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            let rule_filter = super::path_filter::RulePathFilter::new(root, config, rule)?;
            scan(root, &opts, &files, &target_roots, &rule_filter)
        })
        .collect();
    merge(all)
}

fn merge(all: Result<Vec<Vec<RuleFinding>>>) -> Result<Vec<RuleFinding>> {
    let mut v: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut v);
    Ok(v)
}

fn scan(
    root: &Path,
    opts: &Options,
    files: &[PathBuf],
    target_roots: &[PathBuf],
    rule_filter: &super::path_filter::RulePathFilter,
) -> Result<Vec<RuleFinding>> {
    let shell_candidates = filtered_shell_files(root, opts, files, target_roots, rule_filter);
    if shell_candidates.is_empty() {
        return Ok(Vec::new());
    }
    run_shellcheck(root, opts, &shell_candidates)
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

    if !VALID_SEVERITIES.contains(&sev) {
        return Err(anyhow::anyhow!(
            "invalid shellcheck severity: \"{}\". Expected one of: error, warning, info, style",
            sev
        ));
    }

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
    let re = GCC_RE.get_or_init(|| {
        regex::Regex::new(r"^(.+):\d+:\d+: [a-z]+: ").expect("failed to compile GCC_RE regex")
    });
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
