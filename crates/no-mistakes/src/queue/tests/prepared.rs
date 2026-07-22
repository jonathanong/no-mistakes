use super::*;

#[test]
fn basic_project_reports_queue_edges() {
    let report = analyze_project(&fixture("basic"), None, &[]).unwrap();
    assert_eq!(report.check, vec![]);
    assert!(report
        .edges
        .iter()
        .any(|edge| edge.from == "enqueue.ts" && edge.to == "queues.ts#sendWelcome"));
    assert!(report
        .edges
        .iter()
        .any(|edge| edge.from == "queues.ts#sendWelcome" && edge.to == "worker.ts"));
}

#[test]
fn indexed_prepared_facts_report_matches_plain_prepared_facts_report() {
    let root = fixture("basic");
    let files = crate::codebase::ts_source::discover_files(&root, &[]);
    let facts = crate::codebase::check_facts::collect_check_facts(
        &root,
        files,
        crate::codebase::check_facts::CheckFactPlan {
            queue: true,
            ..Default::default()
        },
    );
    let tsconfig =
        crate::codebase::ts_resolver::resolve_tsconfig_from_visible(None, &root, facts.files())
            .unwrap();

    let plain = analyze_project_with_prepared_facts(&root, &tsconfig, &[], &facts).unwrap();
    let indexed =
        analyze_project_with_prepared_facts_indexed(&root, &tsconfig, &[], &facts).unwrap();

    assert_eq!(indexed.report().producers, plain.producers);
    assert_eq!(indexed.report().workers, plain.workers);
    assert_eq!(indexed.report().jobs, plain.jobs);
    assert_eq!(indexed.report().edges, plain.edges);
    assert_eq!(indexed.report().diagnostics, plain.diagnostics);
    assert_eq!(indexed.report().check, plain.check);
}
