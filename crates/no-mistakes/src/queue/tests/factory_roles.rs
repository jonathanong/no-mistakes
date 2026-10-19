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
        kinds.contains(&"queue-cycle"),
        "a producer/worker loop should be reported, check: {:?}",
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
    assert!(
        report.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("enqueue factory queue argument is not a local binding")
        }),
        "a non-binding enqueue argument should be a diagnostic, diagnostics: {:?}",
        report.diagnostics
    );
}

#[test]
fn cycle_walk_visits_a_queue_that_does_not_enqueue() {
    let root = PathBuf::from("/repo");
    let file = root.join("workers/side.ts");
    let queues = root.join("queues.ts");
    let producers = vec![crate::queue::graph_model::InternalProducer {
        site: crate::queue::extract::ProducerSite {
            file: file.clone(),
            line: 4,
            binding: "leaf".to_string(),
            job: Some("ping".to_string()),
            raw_job: None,
            worker_index: Some(0),
            self_dedup: false,
        },
        queue: Some(crate::queue::types::QueueKey {
            queue_file: queues.clone(),
            queue_name: "leaf".to_string(),
        }),
    }];
    let workers = vec![crate::queue::graph_model::InternalWorker {
        site: crate::queue::extract::WorkerSite {
            file,
            line: 2,
            queue_name: Some("side".to_string()),
            queue_name_import: None,
            jobs: vec!["ping".to_string()],
            processor_specifier: None,
            processor_file: None,
            wildcard: false,
        },
        queue: Some(crate::queue::types::QueueKey {
            queue_file: queues,
            queue_name: "side".to_string(),
        }),
    }];
    let findings = super::super::cycles::queue_cycles(&root, &producers, &workers);
    assert!(
        findings.is_empty(),
        "a one-way enqueue is not a cycle: {findings:?}"
    );
}

#[test]
fn object_form_factories_round_trip_and_prefixed_globs_are_judged() {
    let yaml = r#"
version: 2
projects:
  backend:
    root: pkg
    queues:
      enqueues:
        - pkg
        - pkg/already/*.ts
        - nested/*.ts
        - "["
      workers: []
queues:
  factories:
    queue:
      - createQueue
      - callee: makeQueue
        name: 1
    enqueue:
      - callee: createEnqueueFunction
        queue: 0
        job: 1
    worker:
      - callee: createWorker
        name: 0
        processor: 1
"#;
    let config: crate::config::v2::NoMistakesConfig = serde_yaml::from_str(yaml).unwrap();
    let mut roles = config.queues.factories.clone();
    let extra = roles.clone();
    roles.merge(&extra);
    assert_eq!(roles.queue.len(), 2);
    assert_eq!(
        roles
            .queue
            .iter()
            .find(|factory| factory.callee == "makeQueue")
            .unwrap()
            .name,
        1
    );
    let serialized = serde_yaml::to_string(&config).unwrap();
    let reparsed: crate::config::v2::NoMistakesConfig = serde_yaml::from_str(&serialized).unwrap();
    assert_eq!(config.queues.factories, reparsed.queues.factories);

    let list_yaml = "version: 2\nqueues:\n  factories: [createQueue, makeQueue]\n";
    let list_config: crate::config::v2::NoMistakesConfig = serde_yaml::from_str(list_yaml).unwrap();
    let list_serialized = serde_yaml::to_string(&list_config).unwrap();
    assert!(list_serialized.contains("- createQueue"));
    let list_reparsed: crate::config::v2::NoMistakesConfig =
        serde_yaml::from_str(&list_serialized).unwrap();
    assert_eq!(list_config.queues.factories, list_reparsed.queues.factories);

    let root = PathBuf::from("/repo");
    let mut facts = HashMap::new();
    super::super::globs::record_empty_globs(
        &root,
        &config,
        &[
            root.join("pkg/already/job.ts"),
            root.join("outside.ts"),
            PathBuf::from("/other/pkg/already/job.ts"),
        ],
        &mut facts,
    );
    let messages = facts
        .values()
        .flat_map(|file| file.checks.iter().map(|check| check.message.clone()))
        .collect::<Vec<_>>();
    assert!(messages
        .iter()
        .any(|message| message.contains("pkg/nested/*.ts")));
    assert!(messages
        .iter()
        .any(|message| message.contains("queue glob 'pkg'")));
    assert!(!messages
        .iter()
        .any(|message| message.contains("pkg/already")));
    assert!(!messages.iter().any(|message| message.contains('[')));
}
