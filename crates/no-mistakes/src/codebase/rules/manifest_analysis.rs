use super::{analysis_policy::AnalysisDiagnostics, RuleFinding};
use crate::codebase::ts_source::SourceStore;
use crate::config::v2::{schema::RuleDef, NoMistakesConfig};
use anyhow::Result;
use rayon::prelude::*;
use std::path::Path;

mod classification;
pub(super) use classification::ClassifiedFinding;

pub(super) fn collect(
    root: &Path,
    config: &NoMistakesConfig,
    rule_id: &str,
    sources: &SourceStore,
    defer_suppression: bool,
    scan: impl Fn(&RuleDef) -> Result<Vec<ClassifiedFinding>> + Sync,
) -> Result<AnalysisDiagnostics> {
    let batches: Result<Vec<AnalysisDiagnostics>> = config
        .rule_applications(rule_id)
        .into_par_iter()
        .map(|rule| {
            let mut diagnostics = AnalysisDiagnostics::default();
            let policy = config.effective_rule_analysis(rule)?.unanalyzable;
            let mut findings = scan(rule)?;
            if !defer_suppression {
                super::suppression::suppress_domain_findings_with_sources(
                    root,
                    &mut findings,
                    sources,
                    |classified| {
                        let finding = classified.finding();
                        super::suppression::SuppressionTarget {
                            domain: "codebase",
                            rule: &finding.rule,
                            file: &finding.file,
                            line: Some(finding.line),
                            reason: &finding.message,
                            identity: None,
                        }
                    },
                );
            }
            for classified in findings {
                let unanalyzable = matches!(&classified, ClassifiedFinding::Unanalyzable(_));
                let mut finding = classified.into_public();
                if let Some(message) = &rule.message {
                    finding.message = format!("{message} {}", finding.message);
                }
                if unanalyzable {
                    diagnostics.unanalyzable(policy, finding);
                } else {
                    diagnostics.findings.push(finding);
                }
            }
            Ok(diagnostics)
        })
        .collect();
    let mut diagnostics = AnalysisDiagnostics::default();
    for batch in batches? {
        diagnostics.append(batch);
    }
    super::sort_findings(&mut diagnostics.findings);
    super::sort_findings(&mut diagnostics.advisories);
    Ok(diagnostics)
}

pub(super) fn unavailable(
    root: &Path,
    path: &Path,
    rule: &str,
    error: impl std::fmt::Display,
) -> ClassifiedFinding {
    let file = crate::codebase::ts_source::relative_slash_path(root, path);
    ClassifiedFinding::Unanalyzable(RuleFinding {
        rule: rule.to_owned(), file: file.clone(), line: 1,
        message: format!("{file}: cannot verify dependency policy because this manifest could not be read or parsed: {error}. Repair the manifest or restore read access."),
        import: None, target: Some("unanalyzable".to_owned()),
    })
}

#[cfg(test)]
mod tests;
