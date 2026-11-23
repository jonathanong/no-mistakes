use super::{analyze_project, fixture};
use crate::config::v2::schema::JobStateAsRecordConfig;
use crate::config::v2::NoMistakesConfig;
use crate::queue::CheckSeverity;
use std::path::Path;

#[test]
fn job_state_as_record_findings_snapshot() {
    let root = fixture("job-state-as-record");
    let report = analyze_project(&root, None, &[]).unwrap();
    let findings: Vec<_> = report
        .check
        .iter()
        .filter(|finding| finding.kind == "job-state-as-record")
        .collect();
    assert!(!findings.is_empty());
    assert!(findings
        .iter()
        .all(|finding| finding.severity == Some(CheckSeverity::Advisory)));
    assert!(findings
        .iter()
        .all(|finding| finding.file != "admin/dashboard.ts"));
    let rows: Vec<_> = findings
        .iter()
        .map(|finding| {
            serde_json::json!({
                "kind": finding.kind,
                "severity": finding.severity,
                "file": finding.file,
                "line": finding.line,
                "queueFile": finding.queue_file,
                "queueName": finding.queue_name,
                "message": finding.message,
                "sourceLine": source_line(&root, &finding.file, finding.line),
            })
        })
        .collect();
    let finding_count = findings.len();
    let snapshot = serde_json::to_string_pretty(&rows).unwrap();
    insta::assert_snapshot!(snapshot);

    // Suppression directive removes the suppressed finding.
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let mut suppressed = report.check;
    crate::codebase::rules::suppress_domain_findings_with_sources(
        &root,
        &mut suppressed,
        &sources,
        |finding| crate::codebase::rules::SuppressionTarget {
            domain: "queues",
            rule: "queues-check",
            file: &finding.file,
            line: Some(finding.line),
            reason: &finding.message,
            identity: None,
        },
    );
    let remaining: Vec<_> = suppressed
        .iter()
        .filter(|finding| finding.kind == "job-state-as-record")
        .collect();
    assert_eq!(remaining.len() + 1, finding_count);
}

#[test]
fn job_state_as_record_config_round_trip_and_validation() {
    let enabled: NoMistakesConfig =
        serde_yaml::from_str("version: 2\nqueues:\n  jobStateAsRecord: true\n").unwrap();
    assert_eq!(
        enabled.queues.job_state_as_record,
        Some(JobStateAsRecordConfig::default())
    );
    assert!(serde_yaml::to_string(&enabled)
        .unwrap()
        .contains("jobStateAsRecord: true"));

    let disabled: NoMistakesConfig =
        serde_yaml::from_str("version: 2\nqueues:\n  jobStateAsRecord: false\n").unwrap();
    assert!(disabled.queues.job_state_as_record.is_none());
    assert!(!serde_yaml::to_string(&NoMistakesConfig::default())
        .unwrap()
        .contains("jobStateAsRecord"));

    let custom: NoMistakesConfig = serde_yaml::from_str(
        "version: 2\nqueues:\n  jobStateAsRecord:\n    allowPaths: ['apps/admin/**']\n",
    )
    .unwrap();
    assert_eq!(
        custom.queues.job_state_as_record.unwrap().allow_paths,
        vec!["apps/admin/**".to_string()]
    );

    assert!(serde_yaml::from_str::<NoMistakesConfig>(
        "version: 2\nqueues:\n  jobStateAsRecord:\n    allowPaths: ['[unclosed']\n",
    )
    .is_err());
}

fn source_line(root: &Path, file: &str, line: usize) -> String {
    let text = std::fs::read_to_string(root.join(file)).unwrap_or_default();
    text.lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim()
        .to_string()
}
