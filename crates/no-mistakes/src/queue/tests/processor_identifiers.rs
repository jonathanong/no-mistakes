use super::*;

#[test]
fn named_processors_and_copied_const_tokens_keep_their_worker_ownership() {
    let report = analyze_project(&fixture("processor-identifiers"), None, &[]).unwrap();
    let mut jobs: Vec<_> = report
        .check
        .iter()
        .filter(|finding| finding.kind == "self-dedup-enqueue")
        .map(|finding| finding.job.as_deref().unwrap())
        .collect();
    jobs.sort_unstable();
    assert_eq!(
        jobs,
        [
            "arrow-token",
            "constructed-token",
            "copied-dedup",
            "copied-id",
            "function-token",
            "scoped-token"
        ]
    );
    assert!(
        report
            .check
            .iter()
            .any(|finding| finding.kind == "queue-cycle"),
        "named processor enqueues must participate in cycles: {:?}",
        report.check
    );
    assert!(
        !report.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("self-dedup enqueue was not judged")),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn identifier_processor_shadowing_and_mutation_do_not_borrow_another_body() {
    let root = fixture("processor-identifiers");
    let source = std::fs::read_to_string(root.join("workers/named.mts")).unwrap();
    let roles = serde_yaml::from_str::<crate::config::v2::NoMistakesConfig>(
        &std::fs::read_to_string(root.join(".no-mistakes.yml")).unwrap(),
    )
    .unwrap()
    .queues
    .factories;
    let facts = crate::ast::with_program(&root.join("workers/named.mts"), &source, |program, _| {
        crate::queue::extract::extract_program_with_factories(
            &root.join("workers/named.mts"),
            &source,
            program,
            &[],
            &roles,
            &root,
        )
    })
    .unwrap();
    // The shadowed factory is the fourth worker; its parameter is an unknown callable.
    assert!(facts
        .producers
        .iter()
        .all(|producer| producer.worker_index != Some(3)));
    assert_eq!(
        facts
            .producers
            .iter()
            .filter(|producer| producer.worker_index == Some(0))
            .count(),
        2
    );
    let source = std::fs::read_to_string(root.join("workers/mutated.mts")).unwrap();
    let facts =
        crate::ast::with_program(&root.join("workers/mutated.mts"), &source, |program, _| {
            crate::queue::extract::extract_program_with_factories(
                &root.join("workers/mutated.mts"),
                &source,
                program,
                &[],
                &roles,
                &root,
            )
        })
        .unwrap();
    assert_eq!(facts.producers[0].worker_index, None);
    assert!(!facts.producers[0].self_dedup);
    let source = std::fs::read_to_string(root.join("workers/scopes.mts")).unwrap();
    let facts =
        crate::ast::with_program(&root.join("workers/scopes.mts"), &source, |program, _| {
            crate::queue::extract::extract_program_with_factories(
                &root.join("workers/scopes.mts"),
                &source,
                program,
                &[],
                &roles,
                &root,
            )
        })
        .unwrap();
    assert_eq!(facts.producers[0].worker_index, Some(0));
    assert_eq!(facts.producers[1].worker_index, None);
    assert!(!facts.producers[1].self_dedup);
}
