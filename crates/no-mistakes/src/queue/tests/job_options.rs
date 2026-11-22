use super::{analyze_project, fixture};
use crate::config::v2::NoMistakesConfig;
use crate::queue::extract_model::{
    FileFacts, JobOptionEvidence, JobOptionSite, RequiredJobOptions,
};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

#[test]
fn required_job_options_are_reported() {
    let root = fixture("job-options");
    let report = analyze_project(&root, None, &[]).unwrap();
    let findings: Vec<_> = report
        .check
        .iter()
        .filter(|finding| finding.kind == "missing-job-option")
        .collect();
    assert!(findings.iter().any(|finding| {
        finding.job.as_deref() == Some("plain") && finding.message == "job options missing backoff"
    }));
    assert!(findings
        .iter()
        .all(|finding| finding.job.as_deref() != Some("ready")));
    assert!(findings
        .iter()
        .all(|finding| finding.queue_name.as_deref() != Some("orders")));
    assert!(findings.iter().any(|finding| {
        finding.job.as_deref() == Some("pick") && finding.message == "job options missing priority"
    }));
    assert!(!findings
        .iter()
        .any(|finding| finding.job.as_deref() == Some("open")));
    assert!(findings
        .iter()
        .all(|finding| finding.job.as_deref() != Some("red")));
    assert!(findings
        .iter()
        .all(|finding| finding.job.as_deref() != Some("blue")));
    assert!(findings
        .iter()
        .all(|finding| finding.job.as_deref() != Some("ghost")));
    assert!(findings
        .iter()
        .all(|finding| finding.job.as_deref() != Some("folded-inline")));
    assert!(findings
        .iter()
        .all(|finding| finding.job.as_deref() != Some("folded-const")));
    assert!(findings
        .iter()
        .all(|finding| finding.job.as_deref() != Some("kept-opts")));
    assert!(findings
        .iter()
        .all(|finding| finding.job.as_deref() != Some("called-spread")));
    assert!(findings
        .iter()
        .all(|finding| finding.job.as_deref() != Some("hidden-child")));
    assert!(findings.iter().any(|finding| {
        finding.job.as_deref() == Some("folded-child")
            && finding.message == "job options missing backoff"
    }));
    assert!(findings.iter().any(|finding| {
        finding.job.as_deref() == Some("const-child")
            && finding.message == "job options missing backoff"
    }));
    assert!(findings.iter().all(|finding| {
        source_line(&root, &finding.file, finding.line) != "widgets.add(...head, \"shifted\", {});"
    }));
    for job in [
        "mystery-child",
        "both-spreads",
        "aliased",
        "data-alias",
        "open-bag",
        "empty-bag",
        "computed-node",
        "computed-child",
        "shifted",
    ] {
        assert!(
            findings
                .iter()
                .all(|finding| finding.job.as_deref() != Some(job)),
            "{job}"
        );
    }
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.message.starts_with("job options") && diagnostic.file.ends_with("enqueue.ts")
    }));
    let mut rows = Vec::new();
    for finding in findings {
        rows.push(serde_json::json!({
            "kind": finding.kind,
            "file": finding.file,
            "line": finding.line,
            "queueFile": finding.queue_file,
            "queueName": finding.queue_name,
            "job": finding.job,
            "message": finding.message,
            "sourceLine": source_line(&root, &finding.file, finding.line),
        }));
    }
    let mut diagnostics = Vec::new();
    for diagnostic in report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message.starts_with("job options"))
    {
        diagnostics.push(serde_json::json!({
            "file": diagnostic.file,
            "line": diagnostic.line,
            "message": diagnostic.message,
            "sourceLine": source_line(&root, &diagnostic.file, diagnostic.line),
        }));
    }
    let snapshot = serde_json::to_string_pretty(&serde_json::json!({
        "findings": rows,
        "diagnostics": diagnostics,
    }))
    .unwrap();
    insta::assert_snapshot!(snapshot);

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
    let jobs: Vec<_> = suppressed
        .iter()
        .filter(|finding| finding.kind == "missing-job-option")
        .filter_map(|finding| finding.job.clone())
        .collect();
    assert!(jobs.iter().any(|job| job == "kept"));
    assert!(!jobs.iter().any(|job| job == "skipped"));
    assert!(!jobs.iter().any(|job| job == "file-skipped"));
}

#[test]
fn unset_job_options_add_nothing() {
    let root = fixture("job-options-unset");
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(report
        .check
        .iter()
        .all(|finding| finding.kind != "missing-job-option"));
    assert!(report
        .diagnostics
        .iter()
        .all(|diagnostic| !diagnostic.message.contains("job options")));
}

#[test]
fn required_job_options_round_trip_and_stay_off_the_default_document() {
    let yaml = r#"
version: 2
queues:
  requiredJobOptions: [attempts]
  requiredJobOptionsByQueue:
    items: [priority]
    orders: []
"#;
    let config: NoMistakesConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(
        config.queues.required_job_options,
        vec!["attempts".to_string()]
    );
    assert_eq!(
        config.queues.required_job_options_by_queue["items"],
        vec!["priority".to_string()]
    );
    assert!(config.queues.required_job_options_by_queue["orders"].is_empty());
    let rendered = serde_yaml::to_string(&config).unwrap();
    assert!(rendered.contains("requiredJobOptions"));
    assert!(rendered.contains("orders: []"));
    let default_document = serde_yaml::to_string(&NoMistakesConfig::default()).unwrap();
    assert!(!default_document.contains("requiredJobOptions"));
}

#[test]
fn stamp_keeps_a_policy_already_on_the_facts() {
    let policy = RequiredJobOptions {
        global: vec!["attempts".to_string()],
        by_queue: BTreeMap::new(),
    };
    let mut facts = HashMap::new();
    let file = FileFacts {
        required_job_options: Some(policy.clone()),
        ..FileFacts::default()
    };
    facts.insert(PathBuf::from("widgets.ts"), file);
    crate::queue::job_options::stamp_unconfigured(Path::new("/no/such"), &mut facts);
    assert_eq!(
        facts.values().next().unwrap().required_job_options,
        Some(policy)
    );
}

#[test]
fn unresolved_queue_uses_the_global_list_when_no_queue_overrides_exist() {
    let mut facts = HashMap::new();
    let file = FileFacts {
        required_job_options: Some(RequiredJobOptions {
            global: vec!["attempts".to_string()],
            by_queue: BTreeMap::new(),
        }),
        job_option_sites: vec![JobOptionSite {
            line: 3,
            binding: None,
            queue_name: None,
            queue_file: None,
            job: Some("paint".to_string()),
            evidence: JobOptionEvidence::default(),
        }],
        ..FileFacts::default()
    };
    facts.insert(PathBuf::from("/tmp/widgets.ts"), file);
    facts.insert(PathBuf::from("/tmp/other.ts"), FileFacts::default());
    crate::queue::job_options::judge_into_facts(&mut facts);
    let finding = &facts
        .get(Path::new("/tmp/widgets.ts"))
        .unwrap()
        .job_option_findings[0];
    assert_eq!(finding.message, "job options missing attempts");
    assert!(facts
        .get(Path::new("/tmp/other.ts"))
        .unwrap()
        .job_option_findings
        .is_empty());
}

#[test]
fn unresolved_receiver_abstains_without_queue_overrides() {
    let mut facts = HashMap::new();
    let file = FileFacts {
        required_job_options: Some(RequiredJobOptions {
            global: vec!["attempts".to_string()],
            by_queue: BTreeMap::new(),
        }),
        job_option_sites: vec![JobOptionSite {
            line: 4,
            binding: Some("tags".to_string()),
            queue_name: None,
            queue_file: None,
            job: Some("red".to_string()),
            evidence: JobOptionEvidence::default(),
        }],
        ..FileFacts::default()
    };
    facts.insert(PathBuf::from("/tmp/enqueue.ts"), file);
    crate::queue::job_options::judge_into_facts(&mut facts);
    let judged = facts.get(Path::new("/tmp/enqueue.ts")).unwrap();
    assert!(judged.job_option_findings.is_empty());
    assert!(judged.diagnostics.is_empty());
}

fn source_line(root: &Path, file: &str, line: usize) -> String {
    let text = std::fs::read_to_string(root.join(file)).unwrap_or_default();
    text.lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim()
        .to_string()
}
