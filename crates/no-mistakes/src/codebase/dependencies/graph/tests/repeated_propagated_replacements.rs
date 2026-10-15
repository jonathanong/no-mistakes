use super::*;

#[test]
fn propagated_rhs_writes_preserve_only_before_write_snapshots_and_unrelated_members() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(
        "value-flow-propagated-replacement-rhs",
    ));
    let graph = DepGraph::build_with_plan(
        &root,
        &TsConfig {
            dir: root.clone(),
            paths: vec![],
            paths_dir: root.clone(),
            base_url: None,
        },
        GraphBuildPlan {
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    let file = root.join("src/entry.mts");
    #[cfg(feature = "test-instrumentation")]
    {
        let probe = benchmark_probe_replacements();
        let writes: Vec<_> = probe.rhs.iter().filter(|row| row.file == file).collect();
        assert_eq!(writes.len(), 4);
        assert!(!writes[0].resolved && !writes[0].known);
        assert!(writes[1..].iter().all(|row| row.resolved && row.known));
    }
    let source = std::fs::read_to_string(&file).unwrap();
    let sites = graph.call_sites_in_file(&file);
    for (callee, expected, unknown) in [
        ("dest.get", vec!["dest.get"], true),
        ("after", vec![], true),
        ("dest.set", vec!["dest.set"], false),
        (
            "unrelatedDest.get",
            vec!["unrelatedDest.get", "unrelatedSource.get"],
            false,
        ),
        (
            "snapshotDest.get",
            vec!["snapshotDest.get", "snapshotSource.get"],
            false,
        ),
        (
            "functionDest.get",
            vec!["functionDest.get", "functionSource.get"],
            false,
        ),
    ] {
        let matches: Vec<_> = sites
            .iter()
            .filter(|site| site.source_callee == callee)
            .collect();
        let [site] = matches.as_slice() else {
            panic!("{callee}: {matches:#?}")
        };
        assert_eq!(
            site.offset as usize,
            source.find(format!("{callee}(").as_str()).unwrap()
        );
        let mut actual: Vec<_> = site
            .targets()
            .map(|candidate| match candidate.target {
                KnownCallTargetRef::ModuleExport { export_path, .. } => export_path,
                other => panic!("{callee}: {other:?}"),
            })
            .collect();
        actual.sort();
        assert_eq!(actual, expected, "{site:#?}");
        assert_eq!(site.has_unknown_target(), unknown, "{site:#?}");
    }
}
