//! Offline comparison of file-scoped selections with caller-produced execution evidence.
use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::ExitCode;

use super::{AuditArgs, AuditFormat};

mod model;
pub use model::*;

#[cfg(test)]
mod tests;
mod validate;

pub fn audit_test_selection(
    plan: &TestAuditPlanArtifact,
    observations: &TestAuditObservationsArtifact,
) -> Result<TestAuditReport> {
    validate::artifacts(plan, observations)?;
    let selected: BTreeMap<_, _> = plan
        .plan
        .selected_tests
        .iter()
        .map(|test| (test.test_file.as_str(), test))
        .collect();
    let observed: BTreeMap<_, _> = observations
        .tests
        .iter()
        .map(|test| (test.test_file.as_str(), test))
        .collect();
    let symbol_files: BTreeSet<_> = plan
        .changed_symbols
        .iter()
        .map(|symbol| symbol.file.as_str())
        .collect();
    let file_changes: BTreeSet<_> = plan
        .changed_files
        .iter()
        .map(String::as_str)
        .filter(|file| !symbol_files.contains(file))
        .collect();
    let symbol_changes: BTreeSet<_> = plan.changed_symbols.iter().collect();
    let mut report = TestAuditReport {
        provenance: plan.provenance.clone(),
        observed_test_files: observed.len(),
        selected_test_files: selected.len(),
        selected_observed_tests: Vec::new(),
        missed_observed_tests: Vec::new(),
        selected_without_observed_execution: Vec::new(),
        selected_without_observations: Vec::new(),
        selected_with_incomplete_traces: Vec::new(),
        limitations: vec![
            "Provenance is caller-supplied metadata; the audit compares identities without verifying source digests or the current checkout.".into(),
            "No observed misses does not prove selection completeness: unexecuted branches, missing instrumentation, and indirect effects remain outside this evidence.".into(),
            "Comparison is at test-file granularity within the declared scope, not at individual case or runner-project granularity.".into(),
            "Selected tests without observed changed-code execution are investigation candidates, not proof that those tests are unnecessary.".into(),
        ],
    };
    for (file, trace) in &observed {
        let matched_files: Vec<_> = trace
            .executed_files
            .iter()
            .map(String::as_str)
            .filter(|file| file_changes.contains(file))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(str::to_string)
            .collect();
        let matched_symbols: Vec<_> = trace
            .executed_symbols
            .iter()
            .filter(|symbol| symbol_changes.contains(symbol))
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if !matched_files.is_empty() || !matched_symbols.is_empty() {
            let evidence = TestAuditExecutionEvidence {
                test_file: (*file).into(),
                matched_files,
                matched_symbols,
            };
            if selected.contains_key(file) {
                report.selected_observed_tests.push(evidence);
            } else {
                report.missed_observed_tests.push(evidence);
            }
        } else if let Some(test) = selected.get(file) {
            if trace.trace_complete {
                report
                    .selected_without_observed_execution
                    .push(TestAuditSelectionEvidence {
                        test_file: (*file).into(),
                        reasons: test.reasons.clone(),
                    });
            } else {
                report.selected_with_incomplete_traces.push((*file).into());
            }
        }
    }
    report.selected_without_observations = selected
        .keys()
        .filter(|file| !observed.contains_key(*file))
        .map(|file| (*file).into())
        .collect();
    Ok(report)
}

pub(crate) fn read_artifact<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read audit artifact {}", path.display()))?;
    serde_json::from_str(&content)
        .with_context(|| format!("Invalid audit artifact {}", path.display()))
}

pub(crate) fn render(report: &TestAuditReport, format: AuditFormat) -> String {
    if format == AuditFormat::Json {
        return crate::cli::json_string(report);
    }
    let mut output = format!(
        "Observed missed test files: {}\nSelected test files with no observed changed-code execution: {}\nSelected test files with unknown evidence: {}\n",
        report.missed_observed_tests.len(), report.selected_without_observed_execution.len(),
        report.selected_without_observations.len() + report.selected_with_incomplete_traces.len(),
    );
    for missed in &report.missed_observed_tests {
        output.push_str(&format!("Missed: {}\n", missed.test_file));
    }
    for limitation in &report.limitations {
        output.push_str(&format!("Note: {limitation}\n"));
    }
    output
}

pub(crate) fn run(args: AuditArgs) -> Result<ExitCode> {
    let plan = read_artifact(&args.plan)?;
    let observations = read_artifact(&args.observations)?;
    let report = audit_test_selection(&plan, &observations)?;
    println!("{}", render(&report, args.format));
    // Producing an audit is successful even when it finds observed misses.
    Ok(ExitCode::SUCCESS)
}
