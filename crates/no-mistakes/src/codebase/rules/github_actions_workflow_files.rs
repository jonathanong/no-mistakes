use no_mistakes_rule_kernel::finding::RuleFinding;
use super::source_access;
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use rayon::prelude::*;
use serde_yaml::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(super) mod dispatch;
pub(in crate::codebase::rules) mod env_policy;
pub(super) mod job_lines;
pub(in crate::codebase::rules) mod no_poll_wait_job;
pub(in crate::codebase::rules) mod pr_diff_range;
pub(super) mod run_scripts;
pub(super) mod script_line;
pub(super) mod shell_words;
pub(super) mod step_lines;

/// `Some(true)` is an action file. `Some(false)` is a workflow. `None` is ignored.
pub(super) fn gha_kind(rel: &str) -> Option<bool> {
    let yaml = rel.ends_with(".yml") || rel.ends_with(".yaml");
    if !yaml {
        return None;
    }
    // GitHub only runs a workflow file directly under `.github/workflows/`.
    if let Some(name) = rel.strip_prefix(".github/workflows/") {
        if !name.is_empty() && !name.contains('/') {
            return Some(false);
        }
        return None;
    }
    if rel.starts_with(".github/actions/")
        && (rel.ends_with("/action.yml") || rel.ends_with("/action.yaml"))
    {
        return Some(true);
    }
    None
}

pub(super) fn parse_yaml(
    root: &Path,
    path: &Path,
    sources: &SourceStore,
    source: &str,
    rule_id: &str,
    trim_bom: bool,
) -> Result<Arc<Value>, Box<RuleFinding>> {
    sources.parse_loaded_yaml(path, source, trim_bom).map_err(|error| {
        let mut finding = source_access::unanalyzable_source(
            root,
            path,
            rule_id,
            format_args!("YAML could not be parsed ({error})"),
        );
        finding.line = error.line();
        Box::new(finding)
    })
}

pub(super) fn scan_yaml_files(
    root: &Path,
    files: &[PathBuf],
    sources: &SourceStore,
    rule_id: &str,
    select: &(dyn Fn(&str) -> Option<bool> + Sync),
    judge: &(dyn Fn(&str, &str, &Value, bool) -> Vec<RuleFinding> + Sync),
) -> (Vec<RuleFinding>, Vec<RuleFinding>) {
    let chunks: Vec<_> = files.par_iter().filter_map(|path| {
        let rel = relative_slash_path(root, path);
        let action = select(&rel)?;
        let source = super::read_language_source(sources, path)?;
        Some(parse_yaml(root, path, sources, &source, rule_id, true).map(|value| {
            judge(&rel, source.trim_start_matches('\u{feff}'), &value, action)
        }))
    }).collect();
    let mut findings = Vec::new();
    let mut input_failures = Vec::new();
    for chunk in chunks {
        match chunk {
            Ok(rows) => findings.extend(rows),
            Err(finding) => input_failures.push(*finding),
        }
    }
    (findings, input_failures)
}

/// Recover only directive bytes after a failed read; never analyze lossy workflow YAML.
pub(super) fn unreadable_finding(
    error: &std::io::Error,
    defer_suppression: bool,
    finding: impl FnOnce(usize) -> RuleFinding,
) -> Option<RuleFinding> {
    let invalid_utf8 = error
        .get_ref()
        .and_then(|error| error.downcast_ref::<std::string::FromUtf8Error>());
    let line = invalid_utf8.map_or(1, |error| {
        error.as_bytes()[..error.utf8_error().valid_up_to()]
            .iter()
            .filter(|&&byte| byte == b'\n')
            .count()
            + 1
    });
    let mut findings = vec![finding(line)];
    if let Some(error) = invalid_utf8.filter(|_| !defer_suppression) {
        super::suppress_rule_findings_with_source(
            &mut findings,
            &String::from_utf8_lossy(error.as_bytes()),
        );
    }
    findings.pop()
}

pub(in crate::codebase::rules) fn interpolated_expression_end(value: &str) -> Option<usize> {
    let bytes = value.as_bytes();
    let mut index = 0;
    let mut in_string = false;
    while index < bytes.len() {
        match bytes[index] {
            b'\'' => {
                if in_string && bytes.get(index + 1) == Some(&b'\'') {
                    index += 2;
                } else {
                    in_string = !in_string;
                    index += 1;
                }
            }
            b'}' if !in_string && bytes.get(index + 1) == Some(&b'}') => return Some(index),
            _ => index += 1,
        }
    }
    None
}

#[cfg(all(test, any(nm_shard_all, nm_shard_0)))]
mod tests;
