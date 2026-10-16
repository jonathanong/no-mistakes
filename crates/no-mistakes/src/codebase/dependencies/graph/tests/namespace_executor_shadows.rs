use super::*;

#[test]
fn namespace_executor_shadows_keep_call_occurrences_and_canonical_edges() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("query-reached-per-item")
            .with_file_name("namespace-shadow-fixture"),
    );
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
            imports: true,
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    let expected: std::collections::BTreeMap<String, Vec<u32>> =
        serde_json::from_str(&std::fs::read_to_string(root.join("expected.json")).unwrap())
            .unwrap();
    for (entry, lines) in expected {
        let file = root.join(&entry);
        let sites: Vec<_> = graph
            .resolved_call_sites()
            .iter()
            .filter(|site| site.file == file && site.line > 0)
            .collect();
        let actual: Vec<_> = sites
            .iter()
            .filter(|site| {
                site.typed_from
                    .contains(&("./executor.mts".to_string(), "Executor".to_string()))
            })
            .map(|site| site.line)
            .collect();
        assert_eq!(actual, lines, "{entry}: {sites:#?}");
        let mut identities = std::collections::BTreeSet::new();
        for site in sites {
            assert!(
                identities.insert(site.offset),
                "physical call must remain unique: {site:#?}"
            );
        }
    }
    let call = graph
        .resolved_call_sites()
        .iter()
        .find(|site| site.file == root.join("entry.mts") && site.source_callee == "helper")
        .unwrap();
    assert!(
        matches!(&call.target, ResolvedCallTarget::RepositoryFunction { scope, .. } if scope == "helper")
    );
    let node = call.target_node.as_ref().unwrap();
    assert!(graph.edges.edges().iter().any(|edge| edge.kind == EdgeKind::Call && &edge.to == node && matches!(&edge.from, NodeId::Symbol { symbol, .. } if symbol.to_string() == "canonical")));
}
