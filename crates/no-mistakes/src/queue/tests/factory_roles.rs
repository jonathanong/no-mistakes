use super::*;

#[test]
fn configured_factory_roles_resolve_aliases_and_name_dead_jobs() {
    let root = fixture("factory-roles");
    let report = analyze_project(&root, None, &[]).unwrap();
    let kinds = report
        .check
        .iter()
        .map(|finding| finding.kind.as_str())
        .collect::<Vec<_>>();
    assert!(
        report
            .edges
            .iter()
            .any(|edge| { edge.from.contains("enqueues/page") && edge.to.contains("#page") }),
        "page producer should reach the worker, edges: {:?}",
        report.edges
    );
    assert!(
        kinds.contains(&"unmatched-producer"),
        "dead and orphan producers should be unmatched, check: {:?}",
        report.check
    );
    assert!(
        report
            .check
            .iter()
            .any(|finding| finding.kind == "unmatched-worker"
                && finding.job.as_deref() == Some("stale")),
        "stale handler should be unmatched, check: {:?}",
        report.check
    );
    assert!(
        kinds.contains(&"self-dedup-enqueue"),
        "active-job dedup should be reported, check: {:?}",
        report.check
    );
    assert!(
        kinds.contains(&"debounce-ttl"),
        "debounce ttl should be reported, check: {:?}",
        report.check
    );
    assert!(
        kinds.contains(&"enqueue-and-wait"),
        "addAndWait should be reported, check: {:?}",
        report.check
    );
    assert!(
        report.check.iter().any(|finding| {
            finding.kind == "empty-glob" && finding.message.contains("missing/*.ts")
        }),
        "a configured glob that matches nothing should not pass, check: {:?}",
        report.check
    );
}
