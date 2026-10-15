use super::*;

fn controls_graph() -> (PathBuf, DepGraph) {
    replacement_graph("value-flow-repeated-replacement-controls")
}

fn replacement_graph(name: &str) -> (PathBuf, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(name));
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

fn physical_site<'a>(
    graph: &'a DepGraph,
    root: &Path,
    file: &str,
    callee: &str,
) -> &'a ResolvedCallSite {
    let path = root.join(format!("src/{file}.mts"));
    let sites = graph
        .call_sites_in_file(&path)
        .iter()
        .filter(|site| site.source_callee == callee)
        .collect::<Vec<_>>();
    assert_eq!(sites.len(), 1, "{file}: {callee}: {sites:#?}");
    let source = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        sites[0].offset as usize,
        source.find(format!("{callee}(").as_str()).unwrap()
    );
    sites[0]
}

fn assert_targets(site: &ResolvedCallSite, expected: &[&str], unknown: bool) {
    let mut actual = site
        .targets()
        .map(|candidate| match candidate.target {
            KnownCallTargetRef::ModuleExport { export_path, .. } => export_path.to_owned(),
            other => panic!("expected an imported callable identity: {other:?}"),
        })
        .collect::<Vec<_>>();
    actual.sort();
    assert_eq!(actual, expected, "{site:#?}");
    assert_eq!(site.has_unknown_target(), unknown, "{site:#?}");
}

#[test]
fn stored_rhs_snapshot_stays_exact_when_the_live_source_changes_in_the_loop() {
    let (root, graph) = controls_graph();
    assert_targets(
        physical_site(&graph, &root, "snapshot-vs-live", "snapshotDest.get"),
        &["snapshotDest.get", "source.get"],
        false,
    );
    assert_targets(
        physical_site(&graph, &root, "snapshot-vs-live", "liveDest.get"),
        &["liveDest.get"],
        true,
    );
}

#[test]
fn unsupported_rhs_projections_preserve_original_candidates_and_unwritten_members() {
    let (root, graph) = controls_graph();
    for callee in ["nestedDest.get", "dynamicDest.get"] {
        assert_targets(
            physical_site(&graph, &root, "unsupported-rhs", callee),
            &[callee],
            true,
        );
    }
    assert_targets(
        physical_site(&graph, &root, "unsupported-rhs", "computedDest.get"),
        &["computedDest.get", "rhs.get"],
        false,
    );
    for callee in ["nestedDest.set", "dynamicDest.set"] {
        assert_targets(
            physical_site(&graph, &root, "unsupported-rhs", callee),
            &[callee],
            false,
        );
    }
}

#[test]
fn parameter_rhs_does_not_turn_parameter_agreement_into_a_stable_replacement_leaf() {
    let (root, graph) = controls_graph();
    let site = physical_site(&graph, &root, "parameter-rhs", "parameterDest.get");
    assert_eq!(site.caller.as_deref(), Some("runParameter"));
    assert_targets(site, &["parameterDest.get"], true);
    assert_targets(
        physical_site(&graph, &root, "parameter-rhs", "parameterDest.set"),
        &["parameterDest.set"],
        false,
    );
}

#[test]
fn dynamic_member_writes_do_not_supply_definite_prefix_or_entry_targets() {
    let (root, graph) = replacement_graph("value-flow-replacement-uncertain-execution");
    for callee in ["dynamicMatch.get", "dynamicOther.get"] {
        assert_targets(
            physical_site(&graph, &root, "dynamic-prefix", callee),
            &[],
            true,
        );
    }
    assert_targets(
        physical_site(&graph, &root, "dynamic-prefix", "entryReceiver.get"),
        &["entryReceiver.get"],
        true,
    );
    for callee in ["beforeDynamic.get", "restoredReceiver.get"] {
        assert_targets(
            physical_site(&graph, &root, "dynamic-prefix", callee),
            &["first"],
            false,
        );
    }
}

#[test]
fn executable_accessors_decline_only_the_relevant_new_repeat_proof() {
    let (root, graph) = replacement_graph("value-flow-replacement-uncertain-execution");
    #[cfg(feature = "test-instrumentation")]
    {
        let probe = benchmark_probe_replacements();
        let writes: Vec<_> = probe
            .rhs
            .iter()
            .filter(|row| row.file.ends_with("static-getter.mts"))
            .collect();
        // Both entry candidates were prepared from stable exact writes; the
        // accessor admission, rather than an unsupported RHS, blocks the group.
        assert_eq!(writes.len(), 2);
        assert!(writes.iter().all(|row| row.resolved && row.known));
        assert_eq!(probe.opaque_groups, 1);
    }
    for (file, callee, invocation) in [
        ("static-getter", "Mutator.touch", InvocationKind::Get),
        ("static-setter", "Setter.touch", InvocationKind::Set),
    ] {
        let path = root.join(format!("src/{file}.mts"));
        let accessor = graph
            .call_sites_in_file(&path)
            .iter()
            .find(|site| site.source_callee == callee && site.invocation == invocation)
            .unwrap();
        assert_eq!(accessor.target_nodes().count(), 1, "{accessor:#?}");
        assert!(!accessor.has_unknown_target());
        let source = std::fs::read_to_string(path).unwrap();
        assert_eq!(accessor.offset as usize, source.find(callee).unwrap());
    }
    for (file, callee) in [
        ("static-getter", "getterReceiver.get"),
        ("static-setter", "setterReceiver.get"),
    ] {
        assert_targets(physical_site(&graph, &root, file, callee), &[], true);
    }
    assert_targets(
        physical_site(&graph, &root, "static-getter", "safeReceiver.get"),
        &["first", "safeReceiver.get"],
        false,
    );
    assert_targets(
        physical_site(&graph, &root, "static-getter", "getterReceiver.set"),
        &["getterReceiver.set"],
        false,
    );
}
