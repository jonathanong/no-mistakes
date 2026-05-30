use super::*;

#[test]
fn queue_edges_use_precomputed_shared_facts() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("codebase-intel"));
    let tsconfig =
        crate::codebase::ts_resolver::load_tsconfig(&root.join("tsconfig.json")).unwrap();
    let graph_files = GraphFiles::discover(&root);
    let resolver = crate::codebase::ts_resolver::ImportResolver::new(&tsconfig);
    let config_options = graph_config_options(&root);
    let mut forward = EdgeMap::new();
    let mut reverse = EdgeMap::new();
    let send_email = root.join("packages/api/src/send-email.mts");
    let emails = root.join("packages/api/src/emails.mts");
    let processors = root.join("packages/api/src/processors.mts");
    let fact_plan = effective_ts_fact_plan(
        GraphBuildPlan {
            queues: true,
            ..GraphBuildPlan::default()
        },
        config_options.as_ref(),
    );
    let fact_context = ts_fact_context_for_plan(
        &root,
        GraphBuildPlan {
            queues: true,
            ..GraphBuildPlan::default()
        },
    );
    let facts = collect_ts_facts_with_context(graph_files.indexable(), fact_plan, &fact_context);

    add_queue_edges(
        &root,
        &resolver,
        graph_files.indexable(),
        Some(&facts),
        config_options.as_ref(),
        &mut forward,
        &mut reverse,
    );

    assert!(forward
        .get(&NodeId::File(send_email))
        .map(|edges| {
            edges.iter().any(|(node, kind)| {
                matches!(
                    (node, kind),
                    (
                        NodeId::QueueJob { queue_file, job },
                        EdgeKind::QueueEnqueue
                    ) if queue_file == &emails && job == "sendWelcomeEmail"
                )
            })
        })
        .unwrap_or(false));
    let queue_job = NodeId::QueueJob {
        queue_file: emails,
        job: "sendWelcomeEmail".to_string(),
    };
    assert!(forward
        .get(&queue_job)
        .map(|edges| {
            edges.iter().any(|(node, kind)| {
                *kind == EdgeKind::QueueWorker && node.as_file() == Some(processors.as_path())
            })
        })
        .unwrap_or(false));
}

#[test]
fn process_spawn_edges_cover_source_fallback_without_precomputed_facts() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("codebase-intel"));
    let spawner = root.join("packages/api/src/spawn-runner.mts");
    let spawn_target = root.join("packages/api/src/spawn-target.mts");
    let source = std::fs::read_to_string(&spawner).unwrap();

    let edges = collect_process_spawn_edges(&root, None, &[(spawner.clone(), source)], &[]);

    assert!(edges.iter().any(|(from, to, kind)| {
        *kind == EdgeKind::ProcessSpawn
            && from.as_file() == Some(spawner.as_path())
            && to.as_file() == Some(spawn_target.as_path())
    }));
}

#[test]
fn processor_export_kind_accepts_runtime_exports_only() {
    assert!(is_processor_export_kind(&ExportKind::Function));
    assert!(is_processor_export_kind(&ExportKind::Const));
    assert!(is_processor_export_kind(&ExportKind::Let));
    assert!(is_processor_export_kind(&ExportKind::Var));
    assert!(!is_processor_export_kind(&ExportKind::TypeAlias));
    assert!(!is_processor_export_kind(&ExportKind::Interface));
    assert!(!is_processor_export_kind(&ExportKind::Default));
}
