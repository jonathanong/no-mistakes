use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "integration-test-no-mocks";

const DEFAULT_FORBIDDEN_CALLS: &[&str] = &[
    "vi.mock",
    "vi.doMock",
    "vi.importMock",
    "vi.fn",
    "vi.spyOn",
    "vi.stubGlobal",
    "jest.mock",
    "jest.doMock",
];

const DEFAULT_FORBIDDEN_MODULES: &[&str] = &["msw", "nock", "sinon"];

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) forbidden_calls: Vec<String>,
    pub(crate) forbidden_modules: Vec<String>,
}

struct CompiledOptions {
    calls: Vec<(String, Regex)>,
    modules: Vec<(String, Regex)>,
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
            scan(root, &opts, &files)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let compiled = compile_options(opts)?;
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file(root, path, &compiled))
        .collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn compile_options(opts: &Options) -> Result<CompiledOptions> {
    let calls: Vec<&str> = if opts.forbidden_calls.is_empty() {
        DEFAULT_FORBIDDEN_CALLS.to_vec()
    } else {
        opts.forbidden_calls.iter().map(String::as_str).collect()
    };
    let modules: Vec<&str> = if opts.forbidden_modules.is_empty() {
        DEFAULT_FORBIDDEN_MODULES.to_vec()
    } else {
        opts.forbidden_modules.iter().map(String::as_str).collect()
    };

    Ok(CompiledOptions {
        calls: calls
            .into_iter()
            .map(|call| {
                let pattern = call_pattern(call);
                let regex = Regex::new(&pattern).expect("escaped call pattern is valid regex");
                Ok((call.to_string(), regex))
            })
            .collect::<Result<Vec<_>>>()?,
        modules: modules
            .into_iter()
            .map(|module| {
                let pattern = module_pattern(module);
                let regex = Regex::new(&pattern).expect("escaped module pattern is valid regex");
                Ok((module.to_string(), regex))
            })
            .collect::<Result<Vec<_>>>()?,
    })
}

fn call_pattern(call: &str) -> String {
    let mut pieces = call.split('.');
    let first = pieces.next().unwrap_or_default();
    let rest: Vec<&str> = pieces.collect();
    if first.is_empty() || rest.is_empty() {
        return format!(r"\b{}\s*\(", regex::escape(call));
    }
    let member = rest
        .into_iter()
        .map(regex::escape)
        .collect::<Vec<_>>()
        .join(r"\s*\.\s*");
    format!(r"\b{}\s*\.\s*{}\s*\(", regex::escape(first), member)
}

fn module_pattern(module: &str) -> String {
    let module = regex::escape(module);
    format!(
        r#"\bfrom\s+['"]{module}(?:['"/])|\brequire\s*\(\s*['"]{module}(?:['"/])|\bimport\s*\(\s*['"]{module}(?:['"/])"#
    )
}

fn check_file(root: &Path, path: &Path, compiled: &CompiledOptions) -> Vec<RuleFinding> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let rel = relative_slash_path(root, path);
    let mut findings = Vec::new();
    for (index, raw_line) in content.lines().enumerate() {
        let line = raw_line.split("//").next().unwrap_or("").trim();
        if line.starts_with("/*") || line.starts_with('*') {
            continue;
        }
        if let Some(label) = first_match(line, &compiled.calls, &compiled.modules) {
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.clone(),
                line: index + 1,
                message: format!(
                    "{rel}: integration tests must not use mocking libraries (`{label}`); use real dependencies and test helpers instead"
                ),
                import: Some(label),
                target: None,
            });
        }
    }
    findings
}

fn first_match(
    line: &str,
    calls: &[(String, Regex)],
    modules: &[(String, Regex)],
) -> Option<String> {
    calls
        .iter()
        .find(|(_, regex)| regex.is_match(line))
        .map(|(label, _)| label.clone())
        .or_else(|| {
            modules
                .iter()
                .find(|(_, regex)| regex.is_match(line))
                .map(|(label, _)| label.clone())
        })
}

#[cfg(test)]
mod tests;
