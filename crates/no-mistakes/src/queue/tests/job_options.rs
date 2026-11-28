use super::{analyze_project, fixture};
use std::collections::BTreeSet;
use std::path::Path;

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
    for job in ["hidden-child", "mystery-child", "computed-child"] {
        assert!(
            findings.iter().any(|finding| {
                finding.job.as_deref() == Some(job)
                    && finding.message == "job options missing backoff"
            }),
            "{job}"
        );
    }
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
        "both-spreads",
        "aliased",
        "data-alias",
        "open-bag",
        "empty-bag",
        "computed-node",
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
    for job in [
        "defaults-after-known-override",
        "defaults-from-factory-spread",
    ] {
        assert!(
            findings.iter().any(|finding| {
                finding.job.as_deref() == Some(job)
                    && finding.message == "job options missing backoff"
            }),
            "{job}"
        );
    }
    let defaults_diagnostics: BTreeSet<_> = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.file == "default-spreads.ts"
                && diagnostic
                    .message
                    .contains("defaultJobOptions is not statically known")
        })
        .map(|diagnostic| source_line(&root, &diagnostic.file, diagnostic.line))
        .collect();
    assert_eq!(
        defaults_diagnostics,
        BTreeSet::from(
            [
                r#"open.add("defaults-after-unknown", {});"#,
                r#"shadowed.add("defaults-from-shadow", {});"#,
                r#"circular.add("defaults-from-cycle", {});"#,
                r#"mutableQueue.add("defaults-from-mutable", {});"#,
                r#"importedQueue.add("defaults-from-import", {});"#,
            ]
            .map(str::to_owned)
        )
    );
    assert!(findings.iter().all(|finding| !matches!(
        finding.job.as_deref(),
        Some("defaults-from-known-spread" | "defaults-after-known")
    )));
    let scoped_jobs = findings
        .iter()
        .filter(|finding| finding.file == "scoped-options.ts")
        .map(|finding| finding.job.as_deref().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        scoped_jobs,
        ["incomplete-second", "incomplete-first", "outer-opts"]
    );
    let shadowed = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.file == "scoped-options.ts" && diagnostic.message.starts_with("job options")
        })
        .map(|diagnostic| source_line(&root, &diagnostic.file, diagnostic.line))
        .collect::<Vec<_>>();
    assert_eq!(shadowed.len(), 2, "{shadowed:?}");
    assert!(shadowed[0].contains("opaque-shadow"), "{shadowed:?}");
    assert!(shadowed[1].contains("parameter-shadow"), "{shadowed:?}");
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
fn factories_without_options_are_judged_with_queue_defaults() {
    let rows = required_option_rows("factory-no-options.ts");
    assert_eq!(
        rows,
        vec![(
            "bare-factory-missing".to_string(),
            "queues.ts".to_string(),
            "job options missing backoff".to_string(),
        )]
    );
}

#[test]
fn duplicate_queue_names_keep_the_resolved_definition_defaults() {
    let rows = required_option_rows("queue-default-producers.ts");
    assert_eq!(
        rows,
        vec![
            (
                "primary-defaults".to_string(),
                "queue-default-primary.ts".to_string(),
                "job options missing backoff".to_string()
            ),
            (
                "secondary-defaults".to_string(),
                "queue-default-secondary.ts".to_string(),
                "job options missing removeOnComplete".to_string()
            ),
            (
                "through-star".to_string(),
                "queue-default-secondary.ts".to_string(),
                "job options missing removeOnComplete".to_string()
            ),
            (
                "through-alias".to_string(),
                "queue-default-secondary.ts".to_string(),
                "job options missing removeOnComplete".to_string()
            ),
        ]
    );
}

#[test]
fn flow_queue_members_follow_spread_order() {
    let root = fixture("job-options");
    let rows = required_option_rows("flow-queue-spreads.ts");
    assert_eq!(
        rows.iter()
            .map(|(job, _, message)| (job.as_str(), message.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("flow-to-required", "job options missing priority"),
            ("flow-explicit-after", "job options missing priority"),
        ]
    );
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.file == "flow-queue-spreads.ts"
            && diagnostic
                .message
                .ends_with("the queue name did not resolve")
            && source_line(&root, &diagnostic.file, diagnostic.line).contains("flow-unknown-after")
    }));
}

#[test]
fn unresolved_flow_spread_still_judges_later_children() {
    let root = fixture("job-options-unresolved-flow");
    let report = analyze_project(&root, None, &[]).unwrap();
    let jobs = report
        .check
        .iter()
        .filter(|finding| finding.kind == "missing-job-option")
        .map(|finding| finding.job.as_deref().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(jobs, ["opaque-child", "known-child"]);
    let rows = report
        .check
        .iter()
        .filter(|finding| finding.kind == "missing-job-option")
        .map(|finding| {
            serde_json::json!({
                "file": finding.file, "line": finding.line,
                "job": finding.job, "message": finding.message,
                "sourceLine": source_line(&root, &finding.file, finding.line),
            })
        })
        .collect::<Vec<_>>();
    insta::assert_snapshot!(
        "unresolved_flow_children",
        serde_json::to_string_pretty(&rows).unwrap()
    );
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .ends_with("the options object is not statically known")
    }));
}

#[test]
fn recursive_flow_child_with_explicit_opts_is_judged() {
    let root = fixture("job-options");
    let report = analyze_project(&root, None, &[]).unwrap();
    assert!(required_option_rows("flow-cyclic-child.ts").is_empty());
    assert!(report.diagnostics.iter().all(|diagnostic| {
        diagnostic.file != "flow-cyclic-child.ts" || !diagnostic.message.starts_with("job options")
    }));
}

#[test]
fn mutually_recursive_flow_children_with_explicit_opts_are_judged() {
    let root = fixture("job-options");
    let report = analyze_project(&root, None, &[]).unwrap();
    let file = "flow-mutual-child-cycle.ts";
    assert!(report
        .check
        .iter()
        .all(|finding| finding.file != file || finding.kind != "missing-job-option"));
    assert!(report.diagnostics.iter().all(|diagnostic| {
        diagnostic.file != file || !diagnostic.message.starts_with("job options")
    }));
}

fn required_option_rows(file: &str) -> Vec<(String, String, String)> {
    let report = analyze_project(&fixture("job-options"), None, &[]).unwrap();
    report
        .check
        .into_iter()
        .filter_map(|finding| {
            (finding.kind == "missing-job-option" && finding.file == file).then(|| {
                (
                    finding.job.unwrap(),
                    finding.queue_file.unwrap(),
                    finding.message,
                )
            })
        })
        .collect()
}

fn source_line(root: &Path, file: &str, line: usize) -> String {
    let text = std::fs::read_to_string(root.join(file)).unwrap_or_default();
    text.lines()
        .nth(line.saturating_sub(1))
        .unwrap_or("")
        .trim()
        .to_string()
}
