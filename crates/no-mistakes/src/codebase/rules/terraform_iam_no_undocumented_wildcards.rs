use super::analysis_policy::AnalysisDiagnostics;
use no_mistakes_rule_kernel::finding::RuleFinding;
use crate::codebase::analysis_session::AnalysisSession;
use crate::codebase::check_outcome::{CheckFailure, CheckInputFailure};
use crate::codebase::ts_source::{relative_slash_path, SourceStore};
use crate::config::v2::{NoMistakesConfig, Unanalyzable};
use anyhow::Result;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

mod expr;
mod scan;

pub const RULE_ID: &str = "terraform-iam-no-undocumented-wildcards";
const FIX: &str = "Replace the IAM resource wildcard with explicit resource ARNs. If this action requires a wildcard, add a suppression with the reason beside the resources attribute.";
const TF_JSON_UNANALYZABLE: &str = "Cannot statically verify Terraform JSON file (.tf.json) for IAM resource wildcards. Rewrite the IAM policy in native HCL (.tf) with literal resource ARNs, or exclude this file if reviewed separately.";

struct FileApplication<'a> {
    message: Option<&'a str>,
    unanalyzable: Unanalyzable,
}

pub(super) fn check_with_files(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
) -> Result<Vec<RuleFinding>> {
    super::check_with_borrowed_source_store(root, config, files, |root, config, files, sources| {
        diagnostics_prepared(root, config, files, sources, None, false).map(|d| d.findings)
    })
}

#[doc(hidden)]
pub fn diagnostics_prepared(
    root: &Path,
    config: &NoMistakesConfig,
    files: &[PathBuf],
    sources: &SourceStore,
    session: Option<&AnalysisSession>,
    defer_suppression: bool,
) -> Result<AnalysisDiagnostics> {
    let mut selected = BTreeMap::<PathBuf, Vec<FileApplication<'_>>>::new();
    for rule in config.rule_applications(RULE_ID) {
        let unanalyzable = config.effective_rule_analysis(rule)?.unanalyzable;
        for file in super::selected_rule_files_matching_with_sources(
            root,
            config,
            rule,
            files,
            sources,
            is_terraform_file,
        )? {
            selected.entry(file).or_default().push(FileApplication {
                message: rule.message.as_deref(),
                unanalyzable,
            });
        }
    }

    let results: Vec<_> = selected
        .into_par_iter()
        .map(|(file, applications)| {
            let file_name = relative_slash_path(root, &file);
            scan_file(sources, &file, &file_name, &applications)
        })
        .collect();

    let mut diagnostics = AnalysisDiagnostics::default();
    for result in results {
        match result {
            Ok(file_diagnostics) => diagnostics.append(file_diagnostics),
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
    if !defer_suppression {
        super::suppress_rule_findings_with_sources(root, &mut diagnostics.findings, sources);
        super::suppress_rule_findings_with_sources(root, &mut diagnostics.advisories, sources);
    }
    super::sort_findings(&mut diagnostics.findings);
    super::sort_findings(&mut diagnostics.advisories);
    Ok(diagnostics)
}

fn is_terraform_file(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "tf") || is_tf_json(path)
}

fn is_tf_json(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".tf.json"))
}

fn scan_file(
    sources: &SourceStore,
    path: &Path,
    file: &str,
    applications: &[FileApplication<'_>],
) -> Result<AnalysisDiagnostics, CheckInputFailure> {
    let source = sources.read_source_path(path).map_err(|_| {
        CheckInputFailure::source(
            file,
            "Could not read Terraform source. Restore the file and check its read permissions.",
        )
    })?;
    let mut diagnostics = AnalysisDiagnostics::default();
    if is_tf_json(path) {
        for app in applications {
            diagnostics.unanalyzable(
                app.unanalyzable,
                build_finding(file, 1, app.message, TF_JSON_UNANALYZABLE, true),
            );
        }
        return Ok(diagnostics);
    }
    let hits = scan::scan_hcl(&source).map_err(|error| {
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

    let mut previous = 0;
    let mut line = 1;
    for hit in hits {
        line += source.as_bytes()[previous..hit.offset]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count();
        previous = hit.offset;
        for app in applications {
            match &hit.kind {
                scan::ScanHitKind::Wildcard => {
                    diagnostics.findings.push(build_finding(file, line, app.message, FIX, false));
                }
                scan::ScanHitKind::Unanalyzable(reason) => {
                    diagnostics.unanalyzable(
                        app.unanalyzable,
                        build_finding(file, line, app.message, reason.message(), true),
                    );
                }
            }
        }
    }
    Ok(diagnostics)
}

fn build_finding(
    file: &str,
    line: usize,
    prefix: Option<&str>,
    body: &str,
    unanalyzable: bool,
) -> RuleFinding {
    RuleFinding {
        rule: RULE_ID.to_owned(),
        file: file.to_owned(),
        line,
        message: prefix.map_or_else(|| body.to_owned(), |prefix| format!("{prefix}: {body}")),
        import: None,
        target: unanalyzable.then(|| "unanalyzable".to_owned()),
        causes: Vec::new(),
    }
}

#[cfg(all(test, any(nm_shard_all, nm_shard_2)))]
mod tests;
