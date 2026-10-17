use super::*;

#[test]
fn module_provider_guards_preserve_identity_and_runtime_order_controls() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &crate::test_support::rule_fixture_root("query-reached-per-item")
            .with_file_name("provenance-fixture"),
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
            calls: true,
            ..Default::default()
        },
    )
    .unwrap();
    for (file, callee, expected) in [
        ("controls", "primary.get", Some("primary.get")),
        ("controls", "replica.get", None),
        ("controls", "replica.info", Some("replica.info")),
        ("controls", "unrelated.get", Some("unrelated.get")),
        ("controls", "copied", Some("replica.get")),
        ("capture-before", "primary.get", None),
        ("capture-before", "replica.get", Some("replica.get")),
        ("capture-after", "primary.get", Some("primary.get")),
        ("capture-after", "replica.get", None),
        ("uncalled", "primary.get", Some("primary.get")),
        ("uncalled", "replica.get", Some("replica.get")),
        ("escaped", "primary.get", None),
        ("escaped", "replica.get", None),
        ("cycle", "primary.get", None),
        ("cycle", "replica.get", None),
        ("conditional", "primary.get", None),
        ("conditional", "replica.get", None),
        ("reset", "primary.get", None),
        ("reset", "replica.get", Some("replica.get")),
        ("reordered-rebind", "replica.get", None),
        ("reordered-rebind", "before", Some("replica.get")),
        ("reordered-copy", "method", None),
        ("reordered-copy", "before", Some("replica.get")),
        ("eager-class", "replica.get", None),
        ("eager-class", "before", Some("replica.get")),
        ("argument-rebind", "replica.get", None),
    ] {
        let sites = graph.call_sites_in_file(&root.join(format!("src/module-writer-{file}.mts")));
        let site = sites
            .iter()
            .find(|site| site.source_callee == callee)
            .unwrap();
        match expected {
            Some(expected) => assert!(
                matches!(&site.target, ResolvedCallTarget::ModuleExport { export_path, .. } if export_path == expected),
                "{file}: {site:?}"
            ),
            None => assert!(
                site.has_unknown_target() && site.targets().next().is_none(),
                "{file}: {site:?}"
            ),
        }
    }
    let sites = graph.call_sites_in_file(&root.join("src/module-writer-reordered-copy.mts"));
    let incoming = sites
        .iter()
        .find(|site| site.source_callee == "consume")
        .unwrap();
    assert!(graph
        .dependents_of_node(incoming.target_node.as_ref().unwrap())
        .into_iter()
        .flatten()
        .any(|(_, kind)| *kind == EdgeKind::Call));
}

#[test]
#[cfg(feature = "test-instrumentation")]
fn module_provider_preparation_and_activation_do_not_replay_per_read() {
    for (providers, reads, diamond) in [
        (16, 16, false),
        (256, 16, false),
        (256, 128, false),
        (256, 128, true),
    ] {
        let name = format!(
            "providers-{providers}-reads-{reads}{}",
            if diamond { "-diamond" } else { "" }
        );
        let root = crate::codebase::ts_resolver::normalize_path(
            &fixture("module-receiver-provider-scale").join(&name),
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
                calls: true,
                ..Default::default()
            },
        )
        .unwrap();
        let sites: Vec<_> = graph
            .call_sites_in_file(&root.join("src/entry.mts"))
            .iter()
            .filter(|site| site.source_callee == "replica.get")
            .collect();
        assert_eq!(sites.len(), reads);
        assert!(sites
            .iter()
            .all(|site| site.has_unknown_target() && site.targets().next().is_none()));
        let checks = benchmark_probe_module_provider_checks();
        assert!(
            (16..=16 * 4).contains(&checks),
            "{name}: {checks} provider checks must be independent of physical reads"
        );
        let source = std::fs::read_to_string(root.join("src/entry.mts")).unwrap();
        let allocator = oxc_allocator::Allocator::default();
        let parsed =
            oxc_parser::Parser::new(&allocator, &source, oxc_span::SourceType::ts()).parse();
        let facts = crate::codebase::dependencies::extract::extract_import_facts_from_program_with_source_and_demands(&parsed.program, &source, false, false, true);
        assert_eq!(
            facts.value_flow.module_provider_work,
            [1, providers + 1, 2],
            "one root/path provider scan shared by sixteen writers and alias diamonds"
        );
    }
}
