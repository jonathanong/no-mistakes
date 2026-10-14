use super::*;

fn alias_graph(name: &str) -> (std::path::PathBuf, DepGraph) {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture(name));
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
fn whole_receiver_aliases_keep_projection_and_copied_acquisition_boundaries() {
    let (root, graph) = alias_graph("transparent-receiver-alias");
    for (file, callee, expected) in [
        ("named-default", "alias.get", Some("primary.get")),
        ("inline-default", "alias.get", Some("primary.get")),
        ("chain", "alias.get", Some("primary.get")),
        ("nested", "alias.nested.get", Some("primary.nested.get")),
        ("computed", "alias.get", Some("primary.get")),
        ("supplied", "alias.get", Some("primary.get")),
        ("inline-supplied", "alias.get", Some("primary.get")),
        ("mixed-supplied", "alias.get", Some("primary.get")),
        ("snapshot-named", "get", Some("primary.get")),
        ("snapshot-alias", "get", Some("primary.get")),
        ("object-snapshot", "alias.get", Some("primary.get")),
        ("snapshot-unrelated", "get", Some("primary.get")),
        ("mixed-supplied-reverse", "alias.get", Some("primary.get")),
        ("projected-default", "alias.get", None),
        ("fallback-default", "alias.get", None),
        ("crossfile-provider", "alias.get", None),
        ("matching-before", "alias.get", None),
        ("matching-after-alias", "alias.get", None),
        ("dynamic", "alias.get", None),
        ("shadowed-default", "alias.get", None),
        ("mixed-copy", "alias.get", None),
        ("object-copy", "alias.get", None),
        ("copied-member-alias", "alias", None),
        ("captured", "alias.get", None),
        ("async", "alias.get", None),
        ("reassigned", "alias.get", None),
    ] {
        let sites = graph.call_sites_in_file(&root.join(format!("src/{file}.mts")));
        let reads = sites
            .iter()
            .filter(|site| site.source_callee == callee)
            .collect::<Vec<_>>();
        assert!(!reads.is_empty(), "{file}: {sites:?}");
        for site in reads {
            assert_eq!(
                match &site.target {
                    ResolvedCallTarget::ModuleExport { export_path, .. } =>
                        Some(export_path.as_str()),
                    _ => None,
                },
                expected,
                "{file}: {site:?}"
            );
        }
    }
}

#[test]
fn alias_member_reads_share_agreements_across_small_and_large_inputs() {
    for (kind, agreements) in [("named", 1), ("inline", 2)] {
        for count in [1, 64] {
            let (root, graph) =
                alias_graph(&format!("transparent-receiver-alias-cache-{kind}-{count}"));
            let sites = graph.call_sites_in_file(&root.join("src/mixed.mts"));
            for (callee, expected) in [("alias.get", Some("primary.get")), ("alias.set", None)] {
                let reads = sites
                    .iter()
                    .filter(|site| site.source_callee == callee)
                    .collect::<Vec<_>>();
                assert_eq!(reads.len(), count);
                for site in reads {
                    assert_eq!(
                        match &site.target {
                            ResolvedCallTarget::ModuleExport { export_path, .. } =>
                                Some(export_path.as_str()),
                            _ => None,
                        },
                        expected,
                        "{kind} {count}: {site:?}"
                    );
                }
            }
            #[cfg(feature = "test-instrumentation")]
            {
                // Anonymous immediate callees already need two bootstrap cache
                // generations; extra alias/member reads must not add one.
                assert_eq!(benchmark_probe_parameter_agreements(), agreements);
                assert!(benchmark_probe_origin_nodes() <= 128 * 8 + 8);
            }
            #[cfg(not(feature = "test-instrumentation"))]
            let _ = agreements;
        }
    }
}
