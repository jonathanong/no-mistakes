use super::*;

fn graph(name: &str) -> (std::path::PathBuf, DepGraph) {
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
fn inline_receivers_validate_the_member_read_without_losing_escape_guards() {
    let (root, graph) = graph("inline-member-precision");
    for (file, callee, expected) in [
        ("sibling", "client.get", Some("primary.get")),
        ("matching", "client.get", None),
        ("dynamic", "client.get", None),
        ("nested-sibling", "options.client.get", Some("primary.get")),
        ("nested-matching", "options.client.get", None),
        ("default-sibling", "client.get", Some("primary.get")),
        ("default-matching", "client.get", None),
        ("copied-method", "method", Some("primary.get")),
        ("static-computed", "client.get", Some("primary.get")),
        ("default-object", "options.client.get", Some("primary.get")),
        ("default-object-matching", "options.client.get", None),
        ("captured", "client.get", None),
        ("async", "client.get", None),
    ] {
        let sites = graph.call_sites_in_file(&root.join(format!("src/{file}.mts")));
        let reads = sites
            .iter()
            .filter(|site| site.source_callee == callee)
            .collect::<Vec<_>>();
        assert!(!reads.is_empty(), "{file} {callee}: {sites:?}");
        for site in reads {
            assert_eq!(
                match &site.target {
                    ResolvedCallTarget::ModuleExport { export_path, .. } =>
                        Some(export_path.as_str()),
                    _ => None,
                },
                expected,
                "{file} {callee}: {site:?}"
            );
        }
    }
}

#[test]
fn mixed_member_reads_share_one_parameter_agreement() {
    let (root, graph) = graph("inline-member-precision-cache");
    let sites = graph.call_sites_in_file(&root.join("src/mixed.mts"));
    for (callee, expected) in [("client.get", Some("primary.get")), ("client.set", None)] {
        let reads = sites
            .iter()
            .filter(|site| site.source_callee == callee)
            .collect::<Vec<_>>();
        assert_eq!(reads.len(), 64);
        for site in reads {
            assert_eq!(
                match &site.target {
                    ResolvedCallTarget::ModuleExport { export_path, .. } =>
                        Some(export_path.as_str()),
                    _ => None,
                },
                expected,
                "{callee}: {site:?}"
            );
        }
    }
    #[cfg(feature = "test-instrumentation")]
    {
        assert_eq!(benchmark_probe_parameter_agreements(), 1);
        assert!(
            benchmark_probe_origin_nodes() <= 128 * 8 + 8,
            "member uses must share their agreement/origin DAG: {} nodes",
            benchmark_probe_origin_nodes()
        );
    }
}
