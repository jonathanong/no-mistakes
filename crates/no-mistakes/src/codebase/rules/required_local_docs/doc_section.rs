use crate::codebase::rules::{
    analysis_policy::AnalysisDiagnostics,
    file_diagnostics::{classified_applications, ClassifiedInputFindings, FileRuleInputs},
    source_access::read_failure,
    RuleFinding,
};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::{Context, Result};
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
    all_files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    let sources = crate::codebase::rules::source_store_for_files(all_files);
    diagnostics_with_files_and_sources(root, config, all_files, &sources, true)
        .map(|output| output.findings)
}

#[doc(hidden)]
pub fn diagnostics_with_files_and_sources(
    root: &Path,
    config: &NoMistakesConfig,
    all_files: &[PathBuf],
    sources: &SourceStore,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    classified_applications(
        FileRuleInputs {
            root,
            config,
            files: all_files,
            sources,
            defer_suppression,
        },
        REQUIRED_DOC_SECTION_RULE_ID,
        |opts, files, _, _| {
            Ok(ClassifiedInputFindings {
                findings: scan_doc_section_with_sources(root, opts, files, sources)?,
                ..Default::default()
            })
        },
    )
}

pub(super) fn scan_doc_section_with_sources(
    root: &Path,
    opts: &DocSectionOptions,
    files: &[PathBuf],
    sources: &SourceStore,
) -> Result<Vec<RuleFinding>> {
    let Some(glob_set) = compile_options(opts)? else {
        return Ok(Vec::new());
    };
    let mut findings = Vec::new();
    for file in files {
        let rel = relative_slash_path(root, file);
        if !glob_set.is_match(&rel) {
            continue;
        }
        let content = match sources.read_source_path(file) {
            Ok(content) => content,
            Err(error) => {
                findings.push(read_failure(root, file, REQUIRED_DOC_SECTION_RULE_ID, &error));
                continue;
            }
        };
        if !crate::codebase::markdown_sections::has_required_heading(
            &content,
            &opts.required_heading,
        ) {
            findings.push(RuleFinding {
                causes: Vec::new(),
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

pub(in crate::codebase::rules) fn compile_options(
    opts: &DocSectionOptions,
) -> Result<Option<globset::GlobSet>> {
    if opts.glob.is_empty() || opts.required_heading.is_empty() {
        return Ok(None);
    }
    let glob_set = GlobSetBuilder::new()
        .add(
            GlobBuilder::new(&opts.glob)
                .literal_separator(true)
                .build()
                .context("options.glob")?,
        )
        .build()?;
    Ok(Some(glob_set))
}
