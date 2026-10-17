use super::*;

fn receiver_graph(exact: bool) -> (PathBuf, DepGraph) {
    let root = fixture("value-flow-exported-value-receiver");
    let root = if exact {
        root.with_file_name("exact-fixture")
    } else {
        root
    };
    let root = crate::codebase::ts_resolver::normalize_path(&root);
    let config = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph = DepGraph::build_with_plan(
        &root,
        &config,
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    (root, graph)
}

#[test]
fn exact_exported_value_receiver_reproduction_has_no_stale_known_method() {
    let (root, graph) = receiver_graph(true);
    assert_receiver_targets(&graph, &root, "entry", &[("method", None)]);
    assert_physical_acquisitions(&graph, &root, "entry", "acquire", 1);
}

#[test]
fn foreign_live_reads_of_repository_value_receivers_do_not_keep_original_methods() {
    let (root, graph) = receiver_graph(false);
    for (file, expected, acquire) in [
        (
            "entry",
            [
                ("method", None),
                ("sibling", Some("shared.set")),
                ("other", Some("other.get")),
                ("copied", Some("shared.get")),
            ],
            "acquire",
        ),
        (
            "barrel-entry",
            [
                ("barrelMethod", None),
                ("barrelSibling", Some("shared.set")),
                ("barrelOther", Some("other.get")),
                ("copiedBarrel", Some("shared.get")),
            ],
            "acquireBarrel",
        ),
    ] {
        assert_receiver_targets(&graph, &root, file, &expected);
        assert_physical_acquisitions(&graph, &root, file, acquire, 2);
    }
}

fn assert_receiver_targets(
    graph: &DepGraph,
    root: &Path,
    file: &str,
    expected: &[(&str, Option<&str>)],
) {
    let sites = graph.call_sites_in_file(&root.join(format!("src/{file}.mts")));
    for (callee, export) in expected.iter().copied() {
        let matching: Vec<_> = sites
            .iter()
            .filter(|site| site.source_callee == callee)
            .collect();
        assert_eq!(matching.len(), 1, "{callee}: {matching:#?}");
        let site = matching[0];
        if let Some(export) = export {
            let targets: Vec<_> = site.targets().collect();
            assert_eq!(targets.len(), 1, "{site:#?}");
            assert!(
                matches!(targets[0].target, KnownCallTargetRef::ModuleExport { export_path, .. }
                if export_path == export),
                "{site:#?}"
            );
            assert!(!site.has_unknown_target(), "{site:#?}");
        } else {
            assert!(
                site.targets().next().is_none() && site.has_unknown_target(),
                "{site:#?}"
            );
            assert_eq!(site.target_nodes().count(), 0);
        }
    }
}

fn assert_physical_acquisitions(
    graph: &DepGraph,
    root: &Path,
    file: &str,
    acquire: &str,
    count: usize,
) {
    let path = root.join(format!("src/{file}.mts"));
    let sites: Vec<_> = graph
        .call_sites_in_file(&path)
        .iter()
        .filter(|site| site.source_callee == acquire)
        .collect();
    // The ordinary module record and binding-owned initializer record share
    // their authored source; retain both existing ownership projections.
    assert_eq!(sites.len(), 2 * count);
    let ordinary: Vec<_> = sites
        .iter()
        .filter(|site| site.caller_id.is_none())
        .collect();
    assert_eq!(ordinary.len(), count);
    let source = std::fs::read_to_string(&path).unwrap();
    let expected: Vec<_> = source
        .match_indices(&format!("{acquire}()"))
        .map(|(offset, _)| offset as u32)
        .collect();
    assert_eq!(
        ordinary.iter().map(|site| site.offset).collect::<Vec<_>>(),
        expected
    );
    let target = sites[0].target_nodes().next().unwrap();
    assert!(sites
        .iter()
        .all(|site| site.invocation == InvocationKind::Call
            && site.target_nodes().any(|node| node == target)));
    // Physical initializer/loop calls share one canonical Call edge, not copies
    // of that edge manufactured from repeated execution alternatives.
    assert_eq!(
        graph
            .dependencies_of_node(&NodeId::file(path))
            .unwrap()
            .iter()
            .filter(|(node, kind)| node == target && *kind == EdgeKind::Call)
            .count(),
        1
    );
}

#[cfg(feature = "test-instrumentation")]
#[test]
fn exported_receiver_acquisitions_keep_physical_incoming_sources() {
    let (root, graph) = receiver_graph(false);
    let probe = benchmark_probe_replacements();
    assert_eq!(probe.catalog_builds, 1);
    for (file, acquire, helper) in [
        ("entry", "acquire", "helper"),
        ("barrel-entry", "acquireBarrel", "barrel-helper"),
    ] {
        let path = root.join(format!("src/{file}.mts"));
        let site = graph
            .call_sites_in_file(&path)
            .iter()
            .find(|site| site.source_callee == acquire)
            .unwrap();
        let NodeId::Symbol {
            callable_id: Some(owner),
            ..
        } = site.target_nodes().next().unwrap()
        else {
            panic!("missing exact callable")
        };
        let rows: Vec<_> = probe
            .incoming
            .iter()
            .filter(|row| row.caller_file == path && row.callee == acquire)
            .collect();
        assert_eq!(rows.len(), 4, "{acquire}: {rows:#?}");
        let mut source_counts = std::collections::BTreeMap::new();
        for row in rows {
            *source_counts.entry(row.offset).or_insert(0) += 1;
            assert_eq!(row.routed_file, root.join(format!("src/{helper}.mts")));
            assert_eq!(row.routed_owner, owner.0);
            assert!(!row.callback && !row.synthetic_callback);
            assert_eq!(row.physical.len(), 2);
            assert!(row
                .physical
                .iter()
                .all(|physical| physical.invocation == "Call"));
            assert_eq!(
                row.physical
                    .iter()
                    .filter(|physical| physical.caller_id.is_none())
                    .count(),
                1
            );
            let call = row.source_call.as_ref().expect("actual parser call source");
            assert_eq!(call.file, path);
            assert_eq!(call.offset, row.offset);
            assert_eq!(call.end, row.offset + acquire.len() as u32 + 2);
            assert_eq!(call.argument_count, 0);
            assert!(call.argument_ordinals.is_empty());
            assert!(!call.bound && call.spread_at.is_none());
        }
        assert_eq!(source_counts.len(), 2);
        assert!(source_counts.values().all(|count| *count == 2));
    }
}
