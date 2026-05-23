use crate::codebase::rules::RuleFinding;
use crate::codebase::ts_source::relative_slash_path;
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use globset::{GlobBuilder, GlobSetBuilder};
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
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let mut findings = Vec::new();
    for rule in config.rule_applications(REQUIRED_DOC_SECTION_RULE_ID) {
        let opts: DocSectionOptions = rule.rule_options();
        findings.extend(scan_doc_section(root, &opts, files)?);
    }
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
        if !content.contains(opts.required_heading.as_str()) {
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
