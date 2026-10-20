use super::*;

#[test]
fn metadata_text_and_unrelated_switches_do_not_narrow_workers() {
    let report = analyze_project(&fixture("processor-dispatch"), None, &[]).unwrap();
    for name in ["unrelated", "inline", "ambiguous"] {
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
    for name in ["mixed", "computed", "comparison", "cases", "inline-mixed"] {
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
