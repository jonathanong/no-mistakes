use super::RuleFinding;
use crate::codebase::ts_source::{discover_files, relative_slash_path};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use regex::Regex;
use serde::Deserialize;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "github-actions-pinned-hash";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) exclude_paths: Vec<String>,
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
        let skip = super::skip_dir_set(config);
        let files: Vec<PathBuf> = all_files
            .iter()
            .filter(|p| super::file_allowed_by_roots_and_skip(root, &skip, p, &target_roots))
            .cloned()
            .collect();
        let files = super::path_filter::filter_rule_files(root, config, rule, &files)?;
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

pub(crate) fn build_patterns() -> (Regex, Regex, Regex) {
    (
        Regex::new(r"^\s*-?\s*uses:\s+(\S+)(.*)$").expect("uses pattern"),
        Regex::new(r"^[0-9a-f]{40}$").expect("sha pattern"),
        // Allow `# v1`, `# v1.2`, `# v1.2.3`, `# 1.87.0`, etc.
        Regex::new(r"^#\s*v?\d+(?:\.\d+)*(?:[-+][\w.-]+)?\s*$").expect("version comment pattern"),
    )
}

fn scan(root: &Path, opts: &Options, files: &[PathBuf]) -> Result<Vec<RuleFinding>> {
    let exclude_set = build_exclude_globset(&opts.exclude_paths);
    let (uses_re, sha_re, version_re) = build_patterns();
    let findings: Vec<RuleFinding> = files
        .par_iter()
        .flat_map(|path| check_file(path, root, &exclude_set, &uses_re, &sha_re, &version_re))
        .collect();
    Ok(findings)
}

pub(crate) fn check_file(
    path: &Path,
    root: &Path,
    exclude_set: &GlobSet,
    uses_re: &Regex,
    sha_re: &Regex,
    version_re: &Regex,
) -> Vec<RuleFinding> {
    let rel_str = relative_slash_path(root, path);

    if exclude_set.is_match(rel_str.as_str()) {
        return Vec::new();
    }

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !matches!(ext, "yml" | "yaml") {
        return Vec::new();
    }
    let rel = rel_str.as_str();
    if !rel.starts_with(".github/workflows/")
        && !(rel.starts_with(".github/actions/")
            && (rel.ends_with("/action.yml") || rel.ends_with("/action.yaml")))
    {
        return Vec::new();
    }

    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };

    let file = relative_slash_path(root, path);
    let mut findings = Vec::new();

    for (i, line) in content.lines().enumerate() {
        let line_num = i + 1;
        if !line.contains("uses:") {
            continue;
        }
        let Some(caps) = uses_re.captures(line) else {
            continue;
        };
        let uses_value = caps.get(1).map_or("", |m| m.as_str());
        let trailing = caps.get(2).map_or("", |m| m.as_str()).trim();

        if uses_value.starts_with("./")
            || uses_value.starts_with("../")
            || uses_value.starts_with("docker://")
        {
            continue;
        }

        let ref_part = uses_value.rsplit_once('@').map_or("", |(_, r)| r);
        let sha_ok = sha_re.is_match(ref_part);
        let comment_ok = version_re.is_match(trailing);

        if !sha_ok || !comment_ok {
            let reason = if !sha_ok {
                format!("`{ref_part}` is not a 40-char commit SHA")
            } else {
                format!("trailing comment must be `# v1.2.3` or `# 1.87.0`, got `{trailing}`")
            };
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: file.clone(),
                line: line_num,
                message: format!(
                    "{file}:{line_num}: `uses: {uses_value}` — {reason} \
                    (pin with `owner/repo@<40-char-sha> # v1.2.3`)"
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
