use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{GlobBuilder, GlobSetBuilder};
use rayon::prelude::*;
use serde::Deserialize;
use std::path::{Path, PathBuf};

use super::REQUIRED_DOC_SECTION_RULE_ID;

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct DocSectionOptions {
    pub glob: String,
    pub required_heading: String,
}

pub fn check_required_doc_section(
    root: &Path,
    config: &NoMistakesConfig,
) -> Result<Vec<RuleFinding>> {
    let files =
        crate::codebase::ts_source::discover_files(root, &config.filesystem.skip_directories);
    check_required_doc_section_with_files(root, config, &files)
}

pub(crate) fn check_required_doc_section_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let all: Result<Vec<Vec<RuleFinding>>> = config
        .rule_applications(REQUIRED_DOC_SECTION_RULE_ID)
        .into_par_iter()
        .map(|rule| -> Result<Vec<RuleFinding>> {
            let opts: DocSectionOptions = rule.rule_options();
            let target_roots = crate::codebase::rules::target_roots(root, config, rule);
            let files: Vec<PathBuf> = all_files
                .iter()
                .filter(|p| target_roots.iter().any(|r| p.starts_with(r)))
                .cloned()
                .collect();
            scan_doc_section(root, &opts, &files)
        })
        .collect();
    let mut findings: Vec<RuleFinding> = all?.into_iter().flatten().collect();
    crate::codebase::rules::sort_findings(&mut findings);
    Ok(findings)
}

pub(crate) fn scan_doc_section(
    root: &Path,
    opts: &DocSectionOptions,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    if opts.glob.is_empty() || opts.required_heading.is_empty() {
        return Ok(Vec::new());
    }
    let glob_set = GlobSetBuilder::new()
        .add(
            GlobBuilder::new(&opts.glob)
                .literal_separator(true)
                .build()?,
        )
        .build()?;
    let mut findings = Vec::new();
    for file in files {
        let rel = relative_slash_path(root, file);
        if !glob_set.is_match(&rel) {
            continue;
        }
        let content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let heading_text = opts.required_heading.trim_start_matches('#').trim();
        if !crate::codebase::markdown_sections::has_section(&content, heading_text) {
            findings.push(RuleFinding {
                rule: REQUIRED_DOC_SECTION_RULE_ID.to_string(),
                file: rel.clone(),
                line: 1,
                message: format!(
                    "{rel}: missing required heading \"{}\"",
                    opts.required_heading
                ),
                import: None,
                target: None,
            });
        }
    }
    Ok(findings)
}
