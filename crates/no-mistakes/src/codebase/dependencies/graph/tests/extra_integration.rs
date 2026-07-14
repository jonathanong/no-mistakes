#[test]
fn codebase_intel_graph_emits_queue_http_route_test_and_process_edges() {
    let (root, _tsconfig, graph) = codebase_intel();

    let send_email = root.join("packages/api/src/send-email.mts");
    let emails = root.join("packages/api/src/emails.mts");
    let processors = root.join("packages/api/src/processors.mts");
    let worker = root.join("packages/api/src/worker.mts");
    let api_client = root.join("packages/web/src/api-client.tsx");
    let api_index = root.join("packages/api/src/index.mts");
    let spec = root.join("tests/e2e/users.spec.ts");
    let page = root.join("packages/web/app/users/[id]/page.tsx");
    let spawner = root.join("packages/api/src/spawn-runner.mts");
    let spawn_target = root.join("packages/api/src/spawn-target.mts");

    let enqueue = graph.deps_of(
        &[NodeId::File(send_email)],
        None,
        Some(&[EdgeKind::QueueEnqueue].into()),
    );
    assert!(enqueue.iter().any(|entry| {
        matches!(
            &entry.node,
            NodeId::QueueJob { queue_file, job }
                if queue_file == &emails && job == "sendWelcomeEmail"
        )
    }));

    let queue_job = NodeId::QueueJob {
        queue_file: emails,
        job: "sendWelcomeEmail".to_string(),
    };
    let workers = graph.deps_of(&[queue_job], None, Some(&[EdgeKind::QueueWorker].into()));
    assert!(has_file(&workers, &processors));
    assert!(has_file(&workers, &worker));

    let http = graph.deps_of(
        &[NodeId::File(api_client.clone())],
        None,
        Some(&[EdgeKind::HttpCall].into()),
    );
    assert!(has_file(&http, &api_index));

    let route_refs = graph.deps_of(
        &[NodeId::File(api_client)],
        None,
        Some(&[EdgeKind::RouteRef].into()),
    );
    assert!(has_file(&route_refs, &api_index));

    let route_tests = graph.deps_of(
        &[NodeId::File(spec)],
        None,
        Some(&[EdgeKind::RouteTest].into()),
    );
    assert!(has_file(&route_tests, &page));

    let process = graph.deps_of(
        &[NodeId::File(spawner)],
        None,
        Some(&[EdgeKind::ProcessSpawn].into()),
    );
    assert!(has_file(&process, &spawn_target));
}
