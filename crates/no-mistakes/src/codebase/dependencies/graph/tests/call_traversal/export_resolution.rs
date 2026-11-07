use super::*;

#[test]
fn call_export_resolution_keeps_missing_facts_and_nonvisible_stars_unknown() {
    let root = crate::codebase::ts_resolver::normalize_path(&fixture("call-traversal"));
    let source = root.join("src/empty-star-barrel.mts");
    let tsconfig = TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root.clone(),
        base_url: None,
    };
    let graph_files = GraphFiles::from_files(vec![source.clone()]);
    let inputs = call_resolution_inputs(&root, &tsconfig, &graph_files);
    let resolver = ImportResolver::new(&tsconfig);

    let missing_facts = TsFactMap::new();
    let indexes = CallableResolutionIndexes::default();
    assert!(matches!(
        resolve_exported_callable(
            &inputs,
            &missing_facts,
            &resolver,
            &source,
            "missing",
            &indexes,
            &mut ExportResolutionTrace::default(),
        ),
        ExportedCallableResolution::Unknown
    ));
    assert!(matches!(
        indexes
            .exports
            .get(&(source.clone(), "missing".to_string()))
            .as_deref(),
        Some(ExportedCallableResolution::Unknown)
    ));

    let star_facts = TsFactMap::from([(
        source.clone(),
        TsFileFacts {
            star_reexport_specifiers: vec!["./empty-star-source.mts".to_string()],
            ..TsFileFacts::default()
        },
    )]);
    assert!(matches!(
        resolve_exported_callable(
            &inputs,
            &star_facts,
            &resolver,
            &source,
            "missing",
            &CallableResolutionIndexes::default(),
            &mut ExportResolutionTrace::default(),
        ),
        ExportedCallableResolution::Unknown
    ));
}

#[test]
fn call_export_resolution_deduplicates_external_stars_without_selecting_collisions() {
    let (root, graph) = call_fixture_graph();
    let consumer = root.join("src/external-star-identities-consumer.mts");
    let sites = graph.call_sites_in_file(&consumer);
    let same = sites
        .iter()
        .find(|site| site.source_callee == "sameOrigin")
        .unwrap();
    assert!(
        matches!(&same.target, ResolvedCallTarget::ModuleExport {
            specifier,
            export_path,
            repository_target: None,
            callable_id: None,
        } if specifier == "vitest" && export_path == "mock"),
        "duplicate star paths must retain the one canonical external export: {same:#?}"
    );
    assert!(same.target_node.is_none(), "{same:#?}");
    for callee in [
        "distinctModules",
        "distinctNames",
        "localCollision",
        "unresolvedStar",
    ] {
        let site = sites
            .iter()
            .find(|site| site.source_callee == callee)
            .unwrap_or_else(|| panic!("missing {callee}: {sites:#?}"));
        assert_eq!(
            site.target,
            ResolvedCallTarget::Unknown,
            "{callee}: {site:#?}"
        );
        assert!(site.target_node.is_none(), "{callee}: {site:#?}");
    }
}
