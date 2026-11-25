use super::{analyze_project, fixture};
use crate::config::v2::schema::{
    JobRetentionKey, QueueIncludeTestsConfig, UnboundedJobRetentionConfig,
};
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

#[test]
fn queue_include_tests_config_round_trip_and_validation() {
    let all: NoMistakesConfig =
        serde_yaml::from_str("version: 2\nqueues:\n  includeTests: true\n").unwrap();
    assert_eq!(all.queues.include_tests, Some(QueueIncludeTestsConfig::All));
    assert!(all.queues.include_tests.as_ref().unwrap().is_all());
    assert!(all
        .queues
        .include_tests
        .as_ref()
        .unwrap()
        .globset()
        .is_none());
    assert!(serde_yaml::to_string(&all)
        .unwrap()
        .contains("includeTests: true"));

    for yaml in [
        "version: 2\nqueues:\n  includeTests: false\n",
        "version: 2\nqueues:\n  includeTests: null\n",
        "version: 2\nqueues:\n  includeTests: []\n",
    ] {
        let cfg: NoMistakesConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(cfg.queues.include_tests.is_none(), "{yaml}");
    }
    assert!(!serde_yaml::to_string(&NoMistakesConfig::default())
        .unwrap()
        .contains("includeTests"));

    let paths: NoMistakesConfig = serde_yaml::from_str(
        "version: 2\nqueues:\n  includeTests: ['tests/orders.test.ts', 'tests/orders.test.ts']\n",
    )
    .unwrap();
    let cfg = paths.queues.include_tests.as_ref().unwrap();
    assert!(!cfg.is_all());
    assert_eq!(
        cfg,
        &QueueIncludeTestsConfig::Paths(vec!["tests/orders.test.ts".to_string()])
    );
    assert!(cfg.globset().is_some());
    assert!(serde_yaml::to_string(&paths)
        .unwrap()
        .contains("tests/orders.test.ts"));

    assert!(serde_yaml::from_str::<NoMistakesConfig>(
        "version: 2\nqueues:\n  includeTests: ['[unclosed']\n",
    )
    .is_err());

    #[derive(serde::Serialize)]
    struct NoneWrap {
        #[serde(
            serialize_with = "crate::config::v2::schema::queues_top::queues_include_tests::serialize_include_tests"
        )]
        v: Option<QueueIncludeTestsConfig>,
    }
    assert!(serde_yaml::to_string(&NoneWrap { v: None }).is_ok());
}

#[test]
fn shared_test_scope_excludes_all_queue_rules_until_opted_in() {
    let root = fixture("shared-test-scope-all-rules");
    let default_report = analyze_project(&root, None, &[]).unwrap();
    assert!(
        default_report
            .check
            .iter()
            .all(|finding| finding.file != "tests/all-rules.test.ts"),
        "configured test scope must exclude every queue rule on tests/all-rules.test.ts: {:#?}",
        default_report.check
    );
    assert_eq!(
        default_report
            .check
            .iter()
            .map(|f| (f.kind.as_str(), f.file.as_str()))
            .collect::<Vec<_>>(),
        vec![("worker-rate-limit-per-process", "src/billing.ts")],
        "test file must not trigger shared-queue-rate-limit or mixed-priority on production queue: {:#?}",
        default_report.check
    );

    let included = crate::queue::analyze_project_with_config(
        &root,
        None,
        Some(&root.join("include-all.yml")),
        &[],
    )
    .unwrap();
    let test_kinds: std::collections::BTreeSet<&str> = included
        .check
        .iter()
        .filter(|finding| finding.file == "tests/all-rules.test.ts")
        .map(|finding| finding.kind.as_str())
        .collect();
    for expected in [
        "invalid-job-id",
        "job-state-as-record",
        "missing-job-option",
        "mixed-priority",
        "queue-cycle",
        "self-dedup-enqueue",
        "shared-queue-rate-limit",
        "unbounded-job-retention",
        "worker-rate-limit-per-process",
    ] {
        assert!(
            test_kinds.contains(expected),
            "includeTests: true must restore {expected} on tests/all-rules.test.ts; got {test_kinds:?}"
        );
    }
}
