use super::*;

fn controls_graph() -> (PathBuf, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "value-flow-repeated-replacement-controls",
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
