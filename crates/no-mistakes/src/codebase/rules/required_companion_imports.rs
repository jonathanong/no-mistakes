mod helpers;

use super::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use helpers::{
    build_companion_globset, build_globset, file_imports, render_template, source_extensions,
    source_info,
};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const RULE_ID: &str = "required-companion-imports";

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct Options {
    pub(crate) source_dirs: Vec<String>,
    pub(crate) source_globs: Vec<String>,
    pub(crate) source_extensions: Vec<String>,
    pub(crate) direct_child_only: bool,
    pub(crate) exclude_basenames: Vec<String>,
    pub(crate) exclude_prefixes: Vec<String>,
    pub(crate) companion_globs: Vec<String>,
    pub(crate) specifier_template: String,
    pub(crate) strip_source_prefix: String,
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
    if opts.companion_globs.is_empty() || opts.specifier_template.is_empty() {
        return Ok(Vec::new());
    }

    let source_globs = build_globset(&opts.source_globs)?;
    let extensions = source_extensions(opts);
    let exclude_basenames: HashSet<&str> =
        opts.exclude_basenames.iter().map(String::as_str).collect();
    let rel_files: Vec<String> = files
        .iter()
        .map(|path| relative_slash_path(root, path))
        .collect();

    let sources = rel_files
        .iter()
        .filter_map(|rel| {
            source_info(
                rel,
                opts,
                source_globs.as_ref(),
                &extensions,
                &exclude_basenames,
            )
        })
        .collect::<Vec<_>>();
    let companion_files = companion_files(opts, &sources, &rel_files)?;

    let mut findings = Vec::new();
    for source in sources
        .iter()
        .filter(|source| !companion_files.contains(&source.rel))
    {
        let companion_globs = build_companion_globset(opts, source)?;
        let companions = rel_files
            .iter()
            .filter(|rel| companion_globs.is_match(rel.as_str()))
            .collect::<Vec<_>>();
        let expected_specifier = render_template(&opts.specifier_template, source);
        if companions.is_empty() {
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: source.rel.clone(),
                line: 1,
                message: format!(
                    "{}: no companion file found importing {}",
                    source.rel, expected_specifier
                ),
                import: None,
                target: Some(expected_specifier),
            });
            continue;
        }

        if !companions
            .iter()
            .any(|rel| file_imports(root, rel, &expected_specifier))
        {
            findings.push(RuleFinding {
                rule: RULE_ID.to_string(),
                file: source.rel.clone(),
                line: 1,
                message: format!(
                    "{}: companion files do not import {}",
                    source.rel, expected_specifier
                ),
                import: None,
                target: Some(expected_specifier),
            });
        }
    }
    findings.sort_by(|a, b| a.file.cmp(&b.file).then(a.message.cmp(&b.message)));
    Ok(findings)
}

fn companion_files(
    opts: &Options,
    sources: &[helpers::SourceInfo],
    rel_files: &[String],
) -> Result<HashSet<String>> {
    let mut companions = HashSet::new();
    for source in sources {
        let globs = build_companion_globset(opts, source)?;
        companions.extend(
            rel_files
                .iter()
                .filter(|rel| globs.is_match(rel.as_str()))
                .cloned(),
        );
    }
    Ok(companions)
}

#[cfg(test)]
mod tests;
