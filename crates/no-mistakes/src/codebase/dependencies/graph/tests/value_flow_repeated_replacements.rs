use super::*;

fn producer_graph() -> (PathBuf, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "value-flow-repeated-receiver-producer",
    ));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    (root, graph)
}
fn producer_selectors(site: &ResolvedCallSite) -> Vec<String> {
    let mut selectors: Vec<_> = site
        .targets()
        .map(|candidate| match candidate.target {
            KnownCallTargetRef::ModuleExport { export_path, .. } => export_path.to_owned(),
            KnownCallTargetRef::RepositoryFunction { scope, .. } => scope.to_owned(),
            KnownCallTargetRef::Global { name } => name.to_owned(),
        })
        .collect();
    selectors.sort();
    selectors
}
fn producer_site<'a>(
    graph: &'a DepGraph,
    root: &Path,
    file: &str,
    callee: &str,
) -> &'a ResolvedCallSite {
    graph
        .call_sites_in_file(&root.join(format!("src/{file}.mts")))
        .iter()
        .find(|site| site.source_callee == callee)
        .unwrap_or_else(|| panic!("missing {file}: {callee}"))
}

#[test]
fn repeated_replacement_prefix_tail_and_closed_owner() {
    let (root, graph) = producer_graph();
    for (file, callee, expected) in [
        (
            "positive",
            "loopReceiver.get",
            vec!["loopReceiver.get", "rhs.get"],
        ),
        ("positive", "prefixReceiver.get", vec!["rhs.get"]),
        (
            "positive",
            "middleReceiver.get",
            vec!["middleReceiver.get", "tail.get"],
        ),
        (
            "closed",
            "closedReceiver.get",
            vec!["closedReceiver.get", "rhs.get"],
        ),
        (
            "controls",
            "dynamicReceiver.get",
            vec!["dynamicReceiver.get", "rhs.get"],
        ),
        ("controls", "onceReceiver.get", vec!["onceReceiver.get"]),
        ("opaque", "savedOpaque", vec!["opaqueBeforeReceiver.get"]),
        ("opaque", "alias", vec!["matchedReceiver.get", "rhs.get"]),
        (
            "opaque",
            "matchedReceiver.get",
            vec!["matchedReceiver.get", "rhs.get"],
        ),
        (
            "opaque-owner",
            "regionOwnerReceiver.get",
            vec!["regionOwnerReceiver.get", "rhs.get"],
        ),
        (
            "opaque",
            "unrelatedOpaqueReceiver.get",
            vec!["rhs.get", "unrelatedOpaqueReceiver.get"],
        ),
    ] {
        let site = producer_site(&graph, &root, file, callee);
        assert_eq!(
            producer_selectors(site),
            expected,
            "{file}: {callee}: {site:#?}"
        );
        assert!(!site.has_unknown_target(), "{site:#?}");
    }
    assert_eq!(
        producer_selectors(producer_site(
            &graph,
            &root,
            "foreign-getter-caller",
            "copied"
        )),
        ["foreignCopiedReceiver.get"]
    );
    assert_eq!(
        producer_selectors(producer_site(
            &graph,
            &root,
            "foreign-getter-caller",
            "sibling"
        )),
        ["foreignGetterReceiver.set"]
    );
    assert_eq!(
        producer_selectors(producer_site(
            &graph,
            &root,
            "foreign-getter-caller",
            "acquiredBefore"
        )),
        ["beforeLoopGetterReceiver.get"]
    );
    let middle = graph.call_sites_in_file(&root.join("src/positive.mts"));
    let reads: Vec<_> = middle
        .iter()
        .filter(|site| site.source_callee == "middleReceiver.get")
        .collect();
    assert_eq!(reads.len(), 2);
    assert_eq!(producer_selectors(reads[1]), ["rhs.get"]);
}

#[test]
fn repeated_replacement_snapshots_keep_acquisition_phase() {
    let (root, graph) = producer_graph();
    for (callee, expected) in [
        ("saved", vec!["rhs.get", "savedReceiver.get"]),
        ("returned", vec!["returnedReceiver.get", "rhs.get"]),
        ("method", vec!["outsideReceiver.get"]),
    ] {
        let site = producer_site(&graph, &root, "snapshots", callee);
        assert_eq!(producer_selectors(site), expected, "{site:#?}");
        assert!(!site.has_unknown_target(), "{site:#?}");
    }
}

#[test]
fn repeated_replacement_unsupported_contexts_cannot_keep_original() {
    let (root, graph) = producer_graph();
    for (file, callee) in [
        ("opaque", "opaqueBeforeReceiver.get"),
        ("opaque", "opaqueAfterReceiver.get"),
        ("opaque", "memberBeforeReceiver.get"),
        ("opaque", "memberAfterReceiver.get"),
        ("opaque", "aliasEscapeReceiver.get"),
        ("opaque", "externalEscapeReceiver.get"),
        ("opaque", "constructorReceiver.get"),
        ("opaque-owner", "closedOpaqueReceiver.get"),
        ("opaque-owner", "nestedOpaqueReceiver.get"),
        ("controls", "unsupportedReceiver.get"),
        ("controls", "nestedReceiver.get"),
        ("controls", "resetReceiver.get"),
        ("controls", "leftResetReceiver.get"),
        ("controls", "destructuredReceiver.get"),
        ("nested-once", "closedOnceReceiver.get"),
        ("nested-once", "unsupportedOnceReceiver.get"),
        ("owners", "escapedReceiver.get"),
        ("owners", "interruptedReceiver.get"),
        ("foreign", "sharedReceiver.get"),
        ("foreign-getter-caller", "fresh"),
        ("repository", "objectReceiver.get"),
    ] {
        let site = producer_site(&graph, &root, file, callee);
        assert_eq!(
            producer_selectors(site),
            Vec::<String>::new(),
            "{file}: {site:#?}"
        );
        assert!(site.has_unknown_target(), "{file}: {site:#?}");
    }
    // Foreign RHS writes invalidate that acquired leaf, without inventing its
    // later value. The destination's original is still one possible target.
    let copied = producer_site(&graph, &root, "foreign", "rhsCopiedReceiver.get");
    assert_eq!(producer_selectors(copied), ["rhsCopiedReceiver.get"]);
    assert!(copied.has_unknown_target());
    let unrelated = producer_site(&graph, &root, "foreign", "unrelatedReceiver.get");
    assert_eq!(
        producer_selectors(unrelated),
        ["rhsSource.set", "unrelatedReceiver.get"]
    );
    assert!(!unrelated.has_unknown_target());
    for (file, callee, expected) in [
        ("foreign", "sharedReceiver.set", "sharedReceiver.set"),
        (
            "nested-once",
            "closedOnceReceiver.set",
            "closedOnceReceiver.set",
        ),
        ("repository", "objectReceiver.set", "objectReceiver.set"),
    ] {
        assert_eq!(
            producer_selectors(producer_site(&graph, &root, file, callee)),
            [expected]
        );
    }
    let saved = producer_site(&graph, &root, "repository", "savedObject");
    assert_eq!(producer_selectors(saved), ["objectReceiver.get"]);
    assert!(!saved.has_unknown_target());
}

#[test]
fn repeated_replacement_repository_nodes_fan_out_once() {
    let (root, graph) = producer_graph();
    let site = producer_site(&graph, &root, "repository", "nodeReceiver.get");
    assert_eq!(
        producer_selectors(site),
        ["nodeReceiver.get", "replacement"]
    );
    let nodes: Vec<_> = site.target_nodes().collect();
    assert_eq!(nodes.len(), 1, "{site:#?}");
    let node = nodes[0];
    let source = NodeId::file(root.join("src/repository.mts"));
    assert!(graph
        .dependencies_of_node(&source)
        .unwrap()
        .iter()
        .any(|(target, kind)| target == node && *kind == EdgeKind::Call));
    let calls = graph.call_sites_in_file(&root.join("src/repository.mts"));
    let direct: Vec<_> = calls
        .iter()
        .filter(|site| site.source_callee == "replacement")
        .collect();
    assert_eq!(direct.len(), 2);
    assert!(direct
        .iter()
        .all(|site| site.target_nodes().any(|target| target == node)));
}

#[cfg(feature = "test-instrumentation")]
fn producer_probe_fixture(name: &str, calls: bool) -> ReplacementProbeSnapshot {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(name));
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let _graph = DepGraph::build_with_plan(
        &root,
        &tsconfig,
        GraphBuildPlan {
            calls,
            imports: true,
            ..Default::default()
        },
    )
    .unwrap();
    benchmark_probe_replacements()
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn repeated_replacement_prepares_states_once_independent_of_read_count() {
    for count in [16, 256] {
        let probe = producer_probe_fixture(&format!("value-flow-replacement-scale-{count}"), true);
        assert_eq!(probe.catalog_builds, 1);
        assert_eq!(probe.hot_file_indexes, 1);
        assert_eq!(probe.receiver_normalizations, 1);
        assert_eq!(probe.canonical_receiver_ids, 1);
        assert_eq!(probe.rhs_attempts, 1);
        assert_eq!(probe.rhs_resolutions, 1);
        assert_eq!(probe.rhs_known, 1);
        assert_eq!(probe.retained_write_indices, 1);
        assert_eq!(probe.region_segments, 1);
        // One singleton prefix and one original+tail payload are prepared.
        assert_eq!(probe.normalized_state_payloads, 2);
        assert_eq!(probe.candidate_leaves, 3);
        assert_eq!(probe.alternatives_payloads, 1);
        assert_eq!(probe.alternatives_candidate_leaves, 2);
        assert_eq!(probe.call_admission_checks, count);
        assert_eq!(probe.region_segment_searches, 1);
        // Immutable member matching is shared; every physical admission stays checked.
        assert_eq!(probe.call_proof_steps, 2);
        assert_eq!(probe.opaque_groups, 0);
        assert_eq!(probe.lookup_count, count);
        assert_eq!(probe.prefix_searches, 2 * count);
        assert_eq!(probe.inherited_frame_qualifications, 0);
        assert_eq!(probe.rhs.len(), 1);
        assert_eq!(probe.rhs[0].write_index, 0);
        assert!(probe.rhs[0].resolved && probe.rhs[0].known);
        assert_eq!(probe.incoming.len(), count);
        assert!(probe.incoming.iter().all(|row| {
            row.callee == "receiver.get"
                && row.physical.len() == 1
                && row.physical[0].invocation == "Call"
                && row
                    .source_call
                    .as_ref()
                    .is_some_and(|call| call.argument_ordinals == [0])
        }));
    }
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn repeated_replacement_incoming_keeps_exact_physical_argument_sources() {
    let (root, graph) = producer_graph();
    let probe = benchmark_probe_replacements();
    let site = producer_site(&graph, &root, "repository", "nodeReceiver.get");
    let NodeId::Symbol {
        file,
        callable_id: Some(owner),
        ..
    } = site.target_nodes().next().unwrap()
    else {
        panic!("missing exact repository target");
    };
    let rows: Vec<_> = probe
        .incoming
        .iter()
        .filter(|row| row.routed_file == file.to_path_buf() && row.routed_owner == owner.0)
        .collect();
    let source = std::fs::read_to_string(root.join("src/repository.mts")).unwrap();
    let expected: Vec<_> = [
        ("nodeReceiver.get(item)", "nodeReceiver.get"),
        ("replacement('one')", "replacement"),
        ("replacement('two')", "replacement"),
    ]
    .into_iter()
    .map(|(expression, callee)| {
        let offset = source
            .find(expression)
            .expect("saved physical source expression") as u32;
        (offset, offset + expression.len() as u32, callee)
    })
    .collect();
    assert_eq!(
        rows.iter()
            .map(|row| (
                row.offset,
                row.source_call.as_ref().unwrap().end,
                row.callee.as_str()
            ))
            .collect::<Vec<_>>(),
        expected
    );
    for row in rows {
        assert_eq!(row.caller_file, root.join("src/repository.mts"));
        assert!(!row.callback && !row.synthetic_callback);
        assert_eq!(row.physical.len(), 1);
        assert_eq!(row.physical[0].caller_id, None);
        assert_eq!(row.physical[0].invocation, "Call");
        let call = row
            .source_call
            .as_ref()
            .expect("incoming must retain its actual parser call source");
        assert_eq!(call.file, row.caller_file);
        assert_eq!(call.offset, row.offset);
        assert!(call.end > call.offset);
        assert_eq!(call.context, None);
        assert!(!call.bound);
        assert_eq!(call.argument_count, 1);
        assert_eq!(call.argument_ordinals, [0]);
        assert_eq!(call.spread_at, None);
    }
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn repeated_replacement_probe_is_request_scoped_and_resets_without_calls() {
    let hot =
        std::thread::spawn(|| producer_probe_fixture("value-flow-replacement-scale-16", true));
    let cold = std::thread::spawn(|| producer_probe_fixture("value-flow-replacement-cold", true));
    assert_eq!(hot.join().unwrap().catalog_builds, 1);
    let cold = cold.join().unwrap();
    assert_eq!(cold.catalog_builds, 0);
    assert_eq!(cold.lookup_count, 0);
    assert!(cold.rhs.is_empty());
    assert_eq!(
        producer_probe_fixture("value-flow-replacement-scale-16", true).catalog_builds,
        1
    );
    let no_calls = producer_probe_fixture("value-flow-replacement-cold", false);
    assert_eq!(no_calls.catalog_builds, 0);
    assert_eq!(no_calls.normalized_state_payloads, 0);
    assert!(no_calls.incoming.is_empty());
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn opaque_module_calls_without_writes_keep_replacement_construction_cold() {
    let probe = producer_probe_fixture("value-flow-replacement-opaque-cold", true);
    assert_eq!(probe.catalog_builds, 0);
    assert_eq!(probe.hot_file_indexes, 0);
    assert_eq!(probe.receiver_normalizations, 0);
    assert_eq!(probe.canonical_receiver_ids, 0);
    assert_eq!(probe.rhs_attempts, 0);
    assert_eq!(probe.retained_write_indices, 0);
    assert_eq!(probe.region_segments, 0);
    assert_eq!(probe.normalized_state_payloads, 0);
    assert_eq!(probe.lookup_count, 0);
    assert_eq!(probe.call_admission_checks, 0);
    assert_eq!(probe.call_proof_steps, 0);
    assert_eq!(probe.opaque_groups, 0);
    assert!(probe.rhs.is_empty());
}
