use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use serde::Deserialize;
use serde_yaml::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "ast-grep-ignore-freshness";

mod entries;
mod globs;
mod lines;
mod scan;
mod select;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    /// Rule YAML files, rule directories, or `sgconfig` files. Empty uses
    /// `sgconfig.yml` / `sgconfig.yaml` at the repository root and their `ruleDirs`.
    pub(crate) files: Vec<String>,
}

pub(crate) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(RULE_ID)
        .into_par_iter()
        .map(|rule| {
            let opts: Options = rule.try_rule_options()?;
            scan(root, config, rule, &opts, all_files)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan(
    root: &Path,
    config: &NoMistakesConfig,
    rule: &crate::config::v2::schema::RuleDef,
    opts: &Options,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let tracked: Vec<String> = all_files
        .iter()
        .filter(|path| path.starts_with(root))
        .map(|path| relative_slash_path(root, path))
        .filter(|relative| !relative.is_empty())
        .collect();
    let tracked_set: BTreeSet<&str> = tracked.iter().map(String::as_str).collect();
    let (parsed, mut findings) = scan::load(root, &opts.files, &tracked, &tracked_set);
    let paths: Vec<PathBuf> = parsed
        .iter()
        .map(|item| root.join(&item.relative))
        .collect();
    let allowed = super::path_filter::filter_rule_files(root, config, rule, &paths)?;
    let allowed_set: BTreeSet<_> = allowed
        .iter()
        .map(|path| relative_slash_path(root, path))
        .collect();
    for item in parsed {
        if allowed_set.contains(&item.relative) {
            findings.extend(ignore_findings(
                &item.relative,
                &item.source,
                &item.documents,
                &item.project,
                &tracked,
            ));
        }
    }
    Ok(findings)
}

fn ignore_findings(
    relative: &str,
    source: &str,
    documents: &[Value],
    project: &str,
    tracked: &[String],
) -> Vec<RuleFinding> {
    entries::ignores_all(documents, source)
        .into_iter()
        .filter_map(|entry| ignore_finding(relative, project, entry, tracked))
        .collect()
}

fn ignore_finding(
    relative: &str,
    project: &str,
    entry: entries::IgnoreEntry,
    tracked: &[String],
) -> Option<RuleFinding> {
    if entry.invalid_shape {
        return Some(finding(
            relative,
            entry.line,
            "ast-grep ignores entry is not a string or {glob, caseInsensitive} mapping, so it matches no tracked path. Use one of those forms.".to_string(),
            Some("ignores".to_string()),
        ));
    }
    match globs::matches_tracked(&entry.pattern, entry.case_insensitive, project, tracked) {
        Ok(true) => None,
        Ok(false) => Some(finding(
            relative,
            entry.line,
            format!(
                "ast-grep ignore `{}` matches no tracked path. Delete the entry or restore a tracked file that matches it.",
                entry.pattern
            ),
            Some(entry.pattern),
        )),
        Err(()) => Some(finding(
            relative,
            entry.line,
            format!(
                "ast-grep ignore `{}` is not a valid glob, so it matches no tracked path. Replace it with a valid ignore glob.",
                entry.pattern
            ),
            Some(entry.pattern),
        )),
    }
}

pub(super) fn finding(
    file: &str,
    line: usize,
    message: String,
    target: Option<String>,
) -> RuleFinding {
    RuleFinding {
        causes: Vec::new(),
        rule: RULE_ID.to_string(),
        file: file.to_string(),
        line,
        message,
        import: None,
        target,
    }
}
