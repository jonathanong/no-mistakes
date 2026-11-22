use super::*;

fn source_line(root: &std::path::Path, file: &str, line: usize) -> String {
    let text = std::fs::read_to_string(root.join(file)).unwrap();
    text.lines()
        .nth(line - 1)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn invalid_rows(root: &std::path::Path) -> Vec<serde_json::Value> {
    let report = analyze_project(root, None, &[]).unwrap();
    assert!(
        report
            .check
            .iter()
            .all(|finding| finding.kind == "invalid-job-id"),
        "other queue findings: {:?}",
        report.check
    );
    report
        .check
        .iter()
        .map(|finding| {
            serde_json::json!({
                "file": finding.file,
                "job": finding.job,
                "line": finding.line,
                "message": finding.message,
                "sourceLine": source_line(root, &finding.file, finding.line),
            })
        })
        .collect()
}

#[test]
fn invalid_job_id_findings_match_snapshot() {
    let root = fixture("invalid-job-id");
    let rows = invalid_rows(&root);
    let sources: Vec<String> = rows
        .iter()
        .filter(|row| row["file"] == "enqueue.ts")
        .map(|row| row["sourceLine"].as_str().unwrap().to_string())
        .collect();
    for absent in [
        r#"jobId: "user-1""#,
        "jobId: id",
        "jobId: `order-${id}`",
        r#"jobId: "item-1""#,
        "jobId: dynamic()",
    ] {
        assert!(
            !sources.iter().any(|line| line.contains(absent)),
            "{absent} was judged, findings: {sources:?}"
        );
    }
    for present in [
        r#"jobId: "user:1""#,
        "order-${id}:retry",
        r#"jobId: "a{b}""#,
        r#"jobId: "a}b""#,
        r#"jobId: "user\u0001""#,
        r#"jobId: "a:{b}""#,
        r#"jobId: "a:{\u0001""#,
        "jobId: widgetId",
        r#"jobId: "spread:1""#,
        r#"jobId: "item:1""#,
        r#"jobId: "order:nightly""#,
        r#"jobId: "order:root""#,
        r#"jobId: "item{child}""#,
        r#"jobId: "skip:me""#,
        r#"jobId: "bare:id""#,
    ] {
        assert!(
            sources.iter().any(|line| line.contains(present)),
            "{present} was not reported, findings: {sources:?}"
        );
    }
    assert!(rows
        .iter()
        .any(|row| row["file"].as_str().unwrap().ends_with("suppressed.ts")));
    let pretty = format!("{}\n", serde_json::to_string_pretty(&rows).unwrap());
    insta::with_settings!({ prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!("invalid_job_id", pretty);
    });
}

#[test]
fn invalid_job_id_respects_queues_check_directives() {
    let root = fixture("invalid-job-id");
    let report = analyze_project(&root, None, &[]).unwrap();
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let sources = crate::codebase::rules::source_store_for_files(&files);
    let mut findings = report.check;
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
    let lines: Vec<String> = findings
        .iter()
        .map(|finding| source_line(&root, &finding.file, finding.line))
        .collect();
    assert!(findings
        .iter()
        .all(|finding| !finding.file.ends_with("suppressed.ts")));
    assert!(!lines.iter().any(|line| line.contains("skip:me")));
    assert!(lines.iter().any(|line| line.contains("bare:id")));
    assert!(lines.iter().any(|line| line.contains(r#"jobId: "user:1""#)));
}

#[test]
fn invalid_job_id_uses_the_lexical_const_and_resolved_queue_receiver() {
    let root = fixture("invalid-job-id");
    let rows = invalid_rows(&root);
    let scoped = rows
        .iter()
        .filter(|row| row["file"] == "scoped-enqueue.ts")
        .map(|row| {
            (
                row["job"].as_str().unwrap(),
                row["message"].as_str().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    for absent in [
        "parameter",
        "mutable",
        "unrelated-add",
        "unrelated-bulk",
        "unrelated-scheduler",
        "unrelated-shadow",
    ] {
        assert!(!scoped.iter().any(|(job, _)| *job == absent), "{scoped:?}");
    }
    assert_eq!(scoped.len(), 3, "{scoped:?}");
    assert_eq!(scoped[0].0, "inner");
    assert!(scoped[0].1.contains("'{' or '}'"), "{scoped:?}");
    assert_eq!(scoped[1].0, "outer");
    assert!(scoped[1].1.contains("':'"), "{scoped:?}");
    assert_eq!(scoped[2].0, "late");
    assert!(scoped[2].1.contains("':'"), "{scoped:?}");
}
