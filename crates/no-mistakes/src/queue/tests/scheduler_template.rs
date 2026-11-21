use super::*;
use std::path::Path;

#[test]
fn job_scheduler_templates_flag_literal_options_and_abstain_when_unknown() {
    let root = fixture("job-scheduler-template");
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(
        report.producers.is_empty(),
        "scheduler templates are not producers: {:?}",
        report.producers
    );
    assert!(
        report.workers.is_empty(),
        "scheduler templates are not workers: {:?}",
        report.workers
    );
    assert!(
        report
            .check
            .iter()
            .all(|finding| finding.kind == "scheduler-template-option"),
        "unexpected queue findings: {:?}",
        report.check
    );
    for file in ["clean.ts", "abstain.ts", "widgets.ts"] {
        assert!(
            !report
                .check
                .iter()
                .any(|finding| finding.file.ends_with(file)),
            "{file} should abstain, check: {:?}",
            report.check
        );
    }
    let mut findings = report.check.clone();
    findings.sort();
    assert!(findings.len() > 4, "{findings:?}");
    for option in ["delay", "deduplication", "parent", "jobId"] {
        assert!(
            findings
                .iter()
                .any(|finding| { finding.message.contains(&format!("`{option}`")) }),
            "{option} should be named, check: {findings:?}"
        );
    }
    insta::assert_snapshot!(render(&root, &findings));

    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    crate::codebase::rules::suppress_domain_findings_with_sources(
        &root,
        &mut findings,
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
    let count = |file: &str| {
        findings
            .iter()
            .filter(|finding| finding.file.ends_with(file))
            .count()
    };
    assert_eq!(count("disabled.ts"), 1);
    assert!(findings.iter().any(|finding| {
        finding.file.ends_with("disabled.ts") && finding.message.contains("`jobId`")
    }));
    assert_eq!(count("file-disabled.ts"), 0);
}

#[test]
fn job_scheduler_factory_config_round_trips_and_rejects_conflicts() {
    let root = fixture("job-scheduler-template");
    let config = crate::config::v2::load_v2_config(&root, None).unwrap();
    let json = serde_json::to_value(&config).unwrap();
    assert_eq!(
        json["queues"]["factories"]["jobScheduler"],
        serde_json::json!([
            { "callee": "scheduleItems", "template": 2 },
            { "callee": "scheduleOrders", "template": 0 }
        ])
    );
    let decoded: crate::config::v2::NoMistakesConfig = serde_json::from_value(json).unwrap();
    assert_eq!(
        decoded.queues.factories.job_scheduler,
        config.queues.factories.job_scheduler
    );
    let mut merged = config.queues.factories.clone();
    merged.merge(&config.queues.factories).unwrap();
    assert_eq!(merged.job_scheduler.len(), 2);

    let conflict = crate::config::v2::load_v2_config(&root, Some(&root.join("conflict.yml")));
    let message = format!("{:#}", conflict.unwrap_err());
    assert!(message.contains("scheduleOrders"), "{message}");
    assert!(message.contains("jobScheduler"), "{message}");

    let left = crate::config::v2::load_v2_config(&root, Some(&root.join("merge-a.yml"))).unwrap();
    let right = crate::config::v2::load_v2_config(&root, Some(&root.join("merge-b.yml"))).unwrap();
    let err = left
        .queues
        .factories
        .clone()
        .merge(&right.queues.factories)
        .unwrap_err();
    assert!(err.contains("scheduleOrders"), "{err}");
    assert!(err.contains("jobScheduler"), "{err}");
}

fn render(root: &Path, findings: &[crate::queue::graph_model::CheckFinding]) -> String {
    let mut lines = Vec::new();
    for finding in findings {
        let source = std::fs::read_to_string(root.join(&finding.file)).unwrap();
        let source_line = source
            .lines()
            .nth(finding.line.saturating_sub(1))
            .unwrap_or("")
            .trim();
        lines.push(format!(
            "{}:{}: {} | {}",
            finding.file, finding.line, finding.message, source_line
        ));
    }
    lines.join("\n")
}
