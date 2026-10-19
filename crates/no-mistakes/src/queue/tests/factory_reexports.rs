use super::*;

#[test]
fn dynamic_producer_is_warning_not_check_failure() {
    let report = analyze_project(&fixture("dynamic"), None, &[]).unwrap();
    assert!(
        report.check.iter().any(|finding| {
            finding.kind == "unmatched-worker-unresolved-producers"
                && finding.message.contains("queues.ts:")
        }),
        "{:?}",
        report.check
    );
    assert!(!report
        .check
        .iter()
        .any(|finding| finding.kind == "unmatched-worker"));
    assert!(report
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.message.contains("dynamic")));
}

#[test]
fn reexports_namespaces_and_imported_jobs_are_not_dead_handlers() {
    let report = analyze_project(&fixture("factory-reexports"), None, &[]).unwrap();
    for job in [
        "vDirect",
        "vAlias2",
        "vChain",
        "vFromStar",
        "vAlias3",
        "vNamespace",
        "vImported",
        "vLocal",
        "vTemplate",
    ] {
        assert!(
            report.edges.iter().any(|edge| {
                edge.kind == EdgeKind::QueueEnqueue && edge.to.ends_with(&format!("#{job}"))
            }),
            "{job} should enqueue, edges: {:?}",
            report.edges
        );
        assert!(
            report.edges.iter().any(|edge| {
                edge.kind == EdgeKind::QueueWorker && edge.from.ends_with(&format!("#{job}"))
            }),
            "{job} should have a worker, edges: {:?}",
            report.edges
        );
        assert!(
            !report
                .check
                .iter()
                .any(|finding| finding.job.as_deref() == Some(job)),
            "{job} should not be unmatched, check: {:?}",
            report.check
        );
    }
    assert!(
        report.check.iter().any(|finding| {
            finding.kind == "unmatched-producer" && finding.job.as_deref() == Some("neverHandled")
        }),
        "imported processor should expose neverHandled, check: {:?}",
        report.check
    );
    assert!(
        report.workers.iter().any(|worker| {
            worker.processor_file.as_deref() == Some("proc.mts")
                && worker.jobs == ["vPlain"]
                && !worker.wildcard
        }),
        "imported processor jobs should come from proc.mts, workers: {:?}",
        report.workers
    );
    assert!(
        report.check.iter().any(|finding| {
            finding.kind == "unmatched-worker-unresolved-producers"
                && finding.job.as_deref() == Some("unproduced")
                && finding.message.contains("enq/e.mts:")
        }),
        "unresolved producer should abstain, check: {:?}",
        report.check
    );
    assert!(
        report.diagnostics.iter().any(|diagnostic| {
            diagnostic.message.contains("dynamic or unresolved")
                && diagnostic.file.contains("enq/e.mts")
        }),
        "substituted template should stay unresolved, diagnostics: {:?}",
        report.diagnostics
    );
    assert!(
        !report
            .edges
            .iter()
            .any(|edge| edge.to.contains("#v${") || edge.to.ends_with("#unproduced")),
        "unresolved jobs should not become edges, edges: {:?}",
        report.edges
    );
}

#[test]
fn star_target_outside_the_fact_set_does_not_invent_a_queue() {
    let report = analyze_project(
        &fixture("factory-reexports"),
        None,
        &["star-out.mts".to_string(), "hop.mts".to_string()],
    )
    .unwrap();
    assert!(
        report
            .edges
            .iter()
            .all(|edge| !edge.from.contains("hidden") && !edge.to.contains("hidden")),
        "a star target excluded from the fact set is unresolved, edges: {:?}",
        report.edges
    );
}
