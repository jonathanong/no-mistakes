use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "no-sparse-checkout";
const DEFAULT_INCLUDE: &[&str] = &[".github/workflows/**", ".github/actions/**"];
const FORBIDDEN_KEYS: &[&str] = &["sparse-checkout", "sparse-checkout-cone-mode"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) include: Vec<String>,
}
struct CompiledOptions {
    include: super::path_filter::GlobMatcher,
}

pub fn check(root: &Path, config: &NoMistakesConfig) -> Result<Vec<RuleFinding>> {
    let files = discover_files(root, &config.filesystem.skip_directories);
    check_with_files(root, config, &files)
}
pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = super::source_store_for_files(files);
    check_with_files_and_sources(root, config, files, &sources)
}

pub(crate) fn check_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    check_with_files_sources_and_deferred_suppression(root, config, all_files, sources, false)
}

pub(crate) fn check_with_files_sources_and_deferred_suppression(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<Vec<RuleFinding>> {
    let results: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            let opts = compile_options(rule.try_rule_options()?)?;
            let roots = super::target_roots(root, config, rule);
            let skip = super::skip_dir_set(config);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|path| super::file_allowed_by_roots_and_skip(root, &skip, path, &roots))
                .filter(|path| {
                    matches!(
                        path.extension().and_then(|value| value.to_str()),
                        Some("yml" | "yaml")
                    )
                })
                .filter(|path| opts.include.is_match(&relative_slash_path(root, path)))
                .cloned()
                .collect();
            let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
            Ok(files
                .par_iter()
                .flat_map(|path| check_file(root, path, sources, defer_suppression))
                .collect())
        })
        .collect();
    let mut findings: Vec<RuleFinding> = results?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}
fn compile_options(options: Options) -> Result<CompiledOptions> {
    let include = if options.include.is_empty() {
        DEFAULT_INCLUDE
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    } else {
        options.include
    };
    Ok(CompiledOptions {
        include: super::path_filter::GlobMatcher::new(
            &include,
            &format!("{RULE_ID} options.include"),
        )?,
    })
}
fn check_file(
    root: &Path,
    path: &Path,
    sources: &SourceStore,
    defer_suppression: bool,
) -> Vec<RuleFinding> {
    let Some(source) = super::read_source(sources, path) else {
        return Vec::new();
    };
    let file = relative_slash_path(root, path);
    if !defer_suppression && crate::codebase::ts_source::has_disable_file_comment(&source, RULE_ID)
    {
        return Vec::new();
    }
    let mut findings = match serde_yaml::from_str::<Value>(&source) {
        Ok(value) => check_document(&file, &source, &value),
        Err(error) => vec![RuleFinding {
            rule: RULE_ID.to_string(),
            file: file.clone(),
            line: error.location().map_or(1, |location| location.line()),
            message: format!("{file}: invalid YAML ({error})"),
            import: None,
            target: None,
        }],
    };
    if !defer_suppression {
        super::suppress_rule_findings_with_source(&mut findings, &source);
    }
    findings
}
fn check_document(file: &str, source: &str, value: &Value) -> Vec<RuleFinding> {
    let mut findings = Vec::new();
    let mut checkout_occurrence = 0;
    if let Some(jobs) = value.get("jobs").and_then(Value::as_mapping) {
        for (job_name, job) in jobs {
            let Some(job) = job.as_mapping() else {
                continue;
            };
            let label = job_name.as_str().unwrap_or("unnamed job");
            findings.extend(check_steps(
                file,
                source,
                job.get("steps").and_then(Value::as_sequence),
                &format!("job \"{label}\""),
                &mut checkout_occurrence,
            ));
        }
    }
    if let Some(runs) = value.get("runs").and_then(Value::as_mapping) {
        findings.extend(check_steps(
            file,
            source,
            runs.get("steps").and_then(Value::as_sequence),
            "composite action",
            &mut checkout_occurrence,
        ));
    }
    findings
}
fn check_steps(
    file: &str,
    source: &str,
    steps: Option<&Vec<Value>>,
    owner: &str,
    checkout_occurrence: &mut usize,
) -> Vec<RuleFinding> {
    let Some(steps) = steps else {
        return Vec::new();
    };
    let mut findings = Vec::new();
    for step in steps.iter().filter_map(Value::as_mapping) {
        if !is_checkout(step) {
            continue;
        }
        let occurrence = *checkout_occurrence;
        *checkout_occurrence += 1;
        let Some(with) = step.get("with").and_then(Value::as_mapping) else {
            continue;
        };
        for key in FORBIDDEN_KEYS {
            if with.contains_key(Value::String((*key).to_string())) {
                let line = checkout_key_line(source, key, occurrence);
                findings.push(RuleFinding { rule: RULE_ID.to_string(), file: file.to_string(), line, message: format!("{file}:{line}: {owner} passes `{key}` to actions/checkout; remove partial-checkout inputs so CI sees the full repository"), import: None, target: Some((*key).to_string()) });
            }
        }
    }
    findings
}
fn is_checkout(step: &Mapping) -> bool {
    step.get("uses")
        .and_then(Value::as_str)
        .is_some_and(|uses| uses.trim().starts_with("actions/checkout@"))
}
fn checkout_key_line(source: &str, key: &str, occurrence: usize) -> usize {
    let lines: Vec<&str> = source.lines().collect();
    let checkout_line = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains("uses:") && line.contains("actions/checkout@"))
        .nth(occurrence)
        .map(|(index, _)| index)
        .unwrap_or(0);
    let step_indent = leading_indent(lines[checkout_line]);
    let mut with_indent = None;
    for (index, line) in lines.iter().enumerate().skip(checkout_line + 1) {
        let indent = leading_indent(line);
        let trimmed = line.trim();
        if trimmed.starts_with("- ") && indent <= step_indent {
            break;
        }
        if with_indent.is_none() && trimmed.starts_with("with:") {
            with_indent = Some(indent);
            continue;
        }
        if let Some(with_indent) = with_indent {
            if !trimmed.is_empty() && indent <= with_indent {
                break;
            }
            if yaml_key_line(trimmed, key) {
                return index + 1;
            }
        }
    }
    checkout_line + 1
}

fn leading_indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

fn yaml_key_line(line: &str, key: &str) -> bool {
    line.starts_with(&format!("{key}:"))
        || line.starts_with(&format!("'{key}':"))
        || line.starts_with(&format!("\"{key}\":"))
}
#[cfg(test)]
mod tests;
