use super::{analyze_project, fixture};
use crate::config::v2::schema::{JobRetentionKey, UnboundedJobRetentionConfig};
use crate::config::v2::NoMistakesConfig;
use std::path::Path;

#[test]
fn unbounded_job_retention_findings_and_dedup_snapshot() {
    let root = fixture("unbounded-retention");
    let report = analyze_project(&root, None, &[]).unwrap();
    let findings: Vec<_> = report
        .check
        .iter()
        .filter(|finding| {
            matches!(
                finding.kind.as_str(),
                "unbounded-job-retention" | "missing-job-option"
            )
        })
        .map(|finding| {
            serde_json::json!({
                "kind": finding.kind,
                "file": finding.file,
                "line": finding.line,
                "queueFile": finding.queue_file,
                "queueName": finding.queue_name,
                "job": finding.job,
                "message": finding.message,
                "sourceLine": source_line(&root, &finding.file, finding.line),
            })
        })
        .collect();
    assert!(
        findings
            .iter()
            .all(|row| row["file"] != "allowed/scripts.ts"),
        "allowPaths must skip allowed/scripts.ts: {findings:?}"
    );
    assert!(
        findings
            .iter()
            .all(|row| !(row["kind"] == "missing-job-option" && row["job"] == "missing-fail")),
        "missing-job-option must not duplicate retention-covered removeOnFail: {findings:?}"
    );
    let snapshot = serde_json::to_string_pretty(&findings).unwrap();
    insta::assert_snapshot!(snapshot);

    // Suppression directive removes the suppressed producer.
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
    assert!(suppressed
        .iter()
        .all(|finding| finding.job.as_deref() != Some("suppressed-job")));
}

#[test]
fn unbounded_job_retention_config_round_trip_and_validation() {
    let enabled: NoMistakesConfig =
        serde_yaml::from_str("version: 2\nqueues:\n  unboundedJobRetention: true\n").unwrap();
    assert_eq!(
        enabled.queues.unbounded_job_retention,
        Some(UnboundedJobRetentionConfig::all())
    );
    assert!(serde_yaml::to_string(&enabled)
        .unwrap()
        .contains("unboundedJobRetention: true"));

    let disabled: NoMistakesConfig =
        serde_yaml::from_str("version: 2\nqueues:\n  unboundedJobRetention: false\n").unwrap();
    assert!(disabled.queues.unbounded_job_retention.is_none());
    let null_cfg: NoMistakesConfig =
        serde_yaml::from_str("version: 2\nqueues:\n  unboundedJobRetention: null\n").unwrap();
    assert!(null_cfg.queues.unbounded_job_retention.is_none());
    assert!(!serde_yaml::to_string(&NoMistakesConfig::default())
        .unwrap()
        .contains("unboundedJobRetention"));
    assert!(enabled
        .queues
        .unbounded_job_retention
        .as_ref()
        .unwrap()
        .allow_globset()
        .is_none());

    let custom: NoMistakesConfig = serde_yaml::from_str(
        "version: 2\nqueues:\n  unboundedJobRetention:\n    require: [removeOnFail, removeOnFail]\n    allowPaths: ['scripts/**']\n",
    )
    .unwrap();
    let cfg = custom.queues.unbounded_job_retention.as_ref().unwrap();
    assert_eq!(cfg.require, vec![JobRetentionKey::RemoveOnFail]);
    assert_eq!(cfg.allow_paths, vec!["scripts/**".to_string()]);
    assert!(cfg.allow_globset().is_some());
    let serialized = serde_yaml::to_string(&custom).unwrap();
    assert!(serialized.contains("removeOnFail"));
    assert!(serialized.contains("scripts/**"));

    assert!(serde_yaml::from_str::<NoMistakesConfig>(
        "version: 2\nqueues:\n  unboundedJobRetention:\n    require: [bogus]\n",
    )
    .is_err());
    assert!(serde_yaml::from_str::<NoMistakesConfig>(
        "version: 2\nqueues:\n  unboundedJobRetention:\n    allowPaths: ['[unclosed']\n",
    )
    .is_err());
    assert!(UnboundedJobRetentionConfig {
        require: Vec::new(),
        allow_paths: vec!["[unclosed".to_string()],
    }
    .allow_globset()
    .is_some());

    #[derive(serde::Serialize)]
    struct NoneWrap {
        #[serde(
            serialize_with = "crate::config::v2::schema::queues_retention::serialize_unbounded_retention"
        )]
        v: Option<UnboundedJobRetentionConfig>,
    }
    assert!(serde_yaml::to_string(&NoneWrap { v: None }).is_ok());

    let mut ev = crate::queue::extract_model::RetentionEvidence::default();
    ev.absorb(
        &crate::queue::extract_model::RetentionEvidence::default(),
        true,
    );
    assert_eq!(
        ev.get(JobRetentionKey::RemoveOnComplete),
        Some(crate::queue::extract_model::RetentionValue::Unknown)
    );
    assert_eq!(
        ev.get(JobRetentionKey::RemoveOnFail),
        Some(crate::queue::extract_model::RetentionValue::Unknown)
    );
}

fn source_line(root: &Path, file: &str, line: usize) -> String {
    let text = std::fs::read_to_string(root.join(file)).unwrap_or_default();
    text.lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim()
        .to_string()
}
