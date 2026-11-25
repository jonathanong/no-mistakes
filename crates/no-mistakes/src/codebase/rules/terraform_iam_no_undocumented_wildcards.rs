use super::RuleFinding;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_outcome::{CheckFailure, CheckInputFailure};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::NoMistakesConfig;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod scan;

pub const RULE_ID: &str = "terraform-iam-no-undocumented-wildcards";
const FIX: &str = "Replace the IAM resource wildcard with explicit resource ARNs. If this action requires a wildcard, add a suppression with the reason beside the resources attribute.";

pub(super) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    super::check_with_borrowed_source_store(root, config, files, |root, config, files, sources| {
        let mut findings = check_prepared(root, config, files, sources, None)?;
        super::suppress_rule_findings_with_sources(root, &mut findings, sources);
        Ok(findings)
    })
}

pub(super) fn check_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    session: Option<&AnalysisSession>,
) -> Result<Vec<RuleFinding>> {
    let mut selected = BTreeMap::<PathBuf, Vec<Option<&str>>>::new();
    for rule in config.rule_applications(RULE_ID) {
        for file in super::selected_rule_files_matching_with_sources(
            root,
            config,
            rule,
            files,
            sources,
            |path| path.extension().is_some_and(|extension| extension == "tf"),
        )? {
            selected
                .entry(file)
                .or_default()
                .push(rule.message.as_deref());
        }
    }

    let results: Vec<_> = selected
        .into_par_iter()
        .map(|(file, messages)| {
            let file_name = relative_slash_path(root, &file);
            scan_file(sources, &file, &file_name, &messages)
        })
        .collect();

    let mut findings = Vec::new();
    for result in results {
        match result {
            Ok(file_findings) => findings.extend(file_findings),
            Err(error) => {
                let Some(session) = session else {
                    return Err(error.into());
                };
                session.record_check_failure(CheckFailure {
                    consumer: RULE_ID.to_owned(),
                    file: error.file,
                    property: error.property,
                    message: error.message,
                });
            }
        }
    }
    super::sort_findings(&mut findings);
    Ok(findings)
}

fn scan_file(
    sources: &SourceStore,
    path: &Path,
    file: &str,
    messages: &[Option<&str>],
) -> Result<Vec<RuleFinding>, CheckInputFailure> {
    let source = sources.read_source_path(path).map_err(|_| {
        CheckInputFailure::source(
            file,
            "Could not read Terraform source. Restore the file and check its read permissions.",
        )
    })?;
    let offsets = scan::wildcard_resources(&source).map_err(|error| {
        let location = error.location();
        CheckInputFailure::source(
            file,
            format!(
                "Could not parse Terraform source at line {}, column {}. Fix the HCL syntax and run the check again.",
                location.line(),
                location.column()
            ),
        )
    })?;

    let mut findings = Vec::new();
    let mut previous = 0;
    let mut line = 1;
    for offset in offsets {
        line += source.as_bytes()[previous..offset]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count();
        previous = offset;
        findings.extend(messages.iter().map(|prefix| RuleFinding {
            rule: RULE_ID.to_owned(),
            file: file.to_owned(),
            line,
            message: prefix.map_or_else(|| FIX.to_owned(), |prefix| format!("{prefix}: {FIX}")),
            import: None,
            target: None,
            causes: Vec::new(),
        }));
    }
    Ok(findings)
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
