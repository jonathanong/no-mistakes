use super::*;
use std::collections::HashSet;

#[test]
fn metadata_text_and_unrelated_switches_do_not_narrow_workers() {
    let report = analyze_project(&fixture("processor-dispatch"), None, &[]).unwrap();
    for name in [
        "unrelated",
        "inline",
        "ambiguous",
        "ambiguous-default",
        "inline-namespace",
        "factory-dynamic",
    ] {
        let worker = report
            .workers
            .iter()
            .find(|worker| worker.queue_name.as_deref() == Some(name))
            .unwrap();
        assert!(worker.wildcard, "{worker:?}");
        assert!(worker.jobs.is_empty(), "{worker:?}");
        assert!(
            !report
                .check
                .iter()
                .any(|finding| finding.queue_name.as_deref() == Some(name)),
            "{:?}",
            report.check
        );
    }
}

#[test]
fn literal_processor_dispatch_remains_precise() {
    let report = analyze_project(&fixture("processor-dispatch"), None, &[]).unwrap();
    for (name, jobs) in [
        ("literal", vec!["alpha", "beta"]),
        ("inline-literal", vec!["alpha"]),
        ("namespace-member", vec!["alpha", "beta"]),
        ("factory-literal", vec!["alpha", "beta"]),
        ("factory-default", vec!["alpha", "beta"]),
    ] {
        let worker = report
            .workers
            .iter()
            .find(|worker| worker.queue_name.as_deref() == Some(name))
            .unwrap();
        assert!(!worker.wildcard, "{worker:?}");
        assert_eq!(worker.jobs, jobs, "{worker:?}");
    }
    assert!(
        report
            .check
            .iter()
            .any(|finding| finding.queue_name.as_deref() == Some("literal")
                && finding.job.as_deref() == Some("unhandled")
                && finding.kind == "unmatched-producer"),
        "{:?}",
        report.check
    );
}

#[test]
fn nonliteral_dispatch_keeps_wildcard_after_a_literal_branch() {
    let report = analyze_project(&fixture("processor-dispatch"), None, &[]).unwrap();
    for name in [
        "mixed",
        "computed",
        "computed-dynamic",
        "comparison",
        "cases",
        "inline-mixed",
    ] {
        let worker = report
            .workers
            .iter()
            .find(|worker| worker.queue_name.as_deref() == Some(name))
            .unwrap();
        assert!(worker.wildcard, "{worker:?}");
        assert_eq!(worker.jobs, ["known"], "{worker:?}");
        assert!(
            !report
                .check
                .iter()
                .any(|finding| finding.queue_name.as_deref() == Some(name)
                    && finding.kind == "unmatched-producer"),
            "{:?}",
            report.check
        );
    }
}

#[test]
fn an_inner_callback_keeps_literal_jobs_and_reports_an_unmatched_producer() {
    let report = analyze_project(&fixture("processor-dispatch"), None, &[]).unwrap();
    let worker = report
        .workers
        .iter()
        .find(|worker| worker.queue_name.as_deref() == Some("callback"))
        .unwrap();
    assert!(!worker.wildcard, "{worker:?}");
    assert_eq!(worker.jobs, ["imp-a"], "{worker:?}");
    assert!(
        report.check.iter().any(|finding| {
            finding.queue_name.as_deref() == Some("callback")
                && finding.job.as_deref() == Some("zzz")
                && finding.kind == "unmatched-producer"
        }),
        "{:?}",
        report.check
    );
}

#[test]
fn a_job_note_outside_a_function_does_not_open_a_dispatch_scope() {
    let mut scopes = HashSet::new();
    crate::queue::extract_processor::record_dispatch_scope(&mut scopes, &[]);
    assert!(scopes.is_empty());
    crate::queue::extract_processor::record_dispatch_scope(&mut scopes, &[12, 40]);
    assert_eq!(scopes, HashSet::from([40]));
}
