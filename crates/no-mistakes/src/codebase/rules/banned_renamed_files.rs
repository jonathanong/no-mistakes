use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "banned-renamed-files";

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct BannedBasename {
    pub(crate) name: String,
    pub(crate) message: String,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) scope: Option<String>,
    pub(crate) banned_basenames: Vec<BannedBasename>,
    pub(crate) extensions: Vec<String>,
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
            scan(root, &opts, &files)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
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
    let mut findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file(path, root, opts))
        .collect();
    findings.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(findings)
}

pub(crate) fn check_file(path: &Path, root: &Path, opts: &Options) -> Vec<RuleFinding> {
    let rel = relative_slash_path(root, path);

    if let Some(scope) = &opts.scope {
        let scope = normalize_scope(scope);
        if !scope.is_empty() {
            let in_scope = rel == scope || rel.starts_with(&format!("{scope}/"));
            if !in_scope {
                return Vec::new();
            }
        }
    }

    let Some(file_name) = path.file_name().and_then(|n| n.to_str()) else {
        return Vec::new();
    };

    // Split into stem and extension
    let (stem, ext) = split_stem_ext(file_name);

    let mut findings = Vec::new();
    for banned in &opts.banned_basenames {
        if stem != banned.name.as_str() {
            continue;
        }
        let dot_ext = format!(".{ext}");
        if opts.extensions.iter().any(|e| e.as_str() == dot_ext) {
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: rel.clone(),
                line: 1,
                message: banned.message.clone(),
                import: None,
                target: None,
            });
        }
    }
    findings
}

fn normalize_scope(scope: &str) -> &str {
    let scope = scope.trim().trim_end_matches('/');
    if scope == "." {
        ""
    } else {
        scope.strip_prefix("./").unwrap_or(scope)
    }
}

fn split_stem_ext(filename: &str) -> (&str, &str) {
    match filename.rfind('.') {
        Some(i) if i > 0 => (&filename[..i], &filename[i + 1..]),
        _ => (filename, ""),
    }
}

#[cfg(test)]
mod tests;
