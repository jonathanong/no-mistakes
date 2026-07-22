#[cfg(unix)]
#[test]
fn scoped_route_helper_aliases_remap_real_targets_to_the_symlink_namespace() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link"),
    );
    let graph_files = GraphFiles::discover(&root);
    let mut catalog_visible = graph_files.all().to_vec();
    catalog_visible.push(root.join("tsconfig.json"));
    let catalog = crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
        &root,
        std::slice::from_ref(&root),
        &catalog_visible,
    );
    let resolver = crate::codebase::ts_resolver::ScopedImportResolver::new(
        &catalog,
        graph_files.visible(),
    );
    let plan = GraphBuildPlan {
        routes: true,
        ..GraphBuildPlan::default()
    };
    let facts = collect_ts_facts_with_context(
        graph_files.indexable(),
        plan.ts_fact_plan(),
        &ts_fact_context_for_plan(&root, plan),
    );
    let client = root.join("src/route-client.ts");

    assert_eq!(
        route_helper_ref_patterns(
            &client,
            facts.get_ts_facts(&client).expect("route client facts"),
            &facts,
            &resolver,
            &graph_files,
        ),
        ["/linked/*"]
    );
}

#[cfg(unix)]
#[test]
fn scoped_symbol_aliases_remap_real_targets_to_the_symlink_namespace() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link"),
    );
    let graph_files = GraphFiles::discover(&root);
    let mut catalog_visible = graph_files.all().to_vec();
    catalog_visible.push(root.join("tsconfig.json"));
    let catalog = crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
        &root,
        std::slice::from_ref(&root),
        &catalog_visible,
    );
    let resolver = crate::codebase::ts_resolver::ScopedImportResolver::new(
        &catalog,
        graph_files.visible(),
    );
    let facts = collect_ts_facts(graph_files.indexable(), TsFactPlan::imports_and_symbols());
    let edges = collect_symbol_edges(
        &root,
        SymbolGraphFiles {
            indexable: graph_files.indexable(),
            all: graph_files.all(),
            visible: graph_files.visible(),
            graph_files: &graph_files,
        },
        &facts,
        &resolver,
        &Default::default(),
        None,
    );
    let client = root.join("src/symbol-client.ts");
    let target = root.join("src/symbol-target.ts");

    assert!(edges.contains(&(
        NodeId::Symbol {
            file: client,
            symbol: "execute".to_string(),
        },
        NodeId::Symbol {
            file: target,
            symbol: "linkedSymbol".to_string(),
        },
        EdgeKind::Import,
    )));
}

#[cfg(unix)]
#[test]
fn scoped_symbol_reexports_and_function_imports_stay_in_the_symlink_namespace() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link"),
    );
    let graph_files = GraphFiles::discover(&root);
    let mut catalog_visible = graph_files.all().to_vec();
    catalog_visible.push(root.join("tsconfig.json"));
    let catalog = crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
        &root,
        std::slice::from_ref(&root),
        &catalog_visible,
    );
    let resolver = crate::codebase::ts_resolver::ScopedImportResolver::new(
        &catalog,
        graph_files.visible(),
    );
    let facts = collect_ts_facts(graph_files.indexable(), TsFactPlan::imports_and_symbols());
    let edges = collect_symbol_edges(
        &root,
        SymbolGraphFiles {
            indexable: graph_files.indexable(),
            all: graph_files.all(),
            visible: graph_files.visible(),
            graph_files: &graph_files,
        },
        &facts,
        &resolver,
        &Default::default(),
        None,
    );
    let target = root.join("src/symbol-target.ts");

    for (source, symbol, kind) in [
        ("src/reexport-direct.ts", "directSymbol", EdgeKind::Import),
        ("src/reexport-star.ts", "linkedSymbol", EdgeKind::Import),
    ] {
        assert!(edges.contains(&(
            NodeId::Symbol {
                file: root.join(source),
                symbol: symbol.to_string(),
            },
            NodeId::Symbol {
                file: target.clone(),
                symbol: "linkedSymbol".to_string(),
            },
            kind,
        )));
    }
    assert!(edges.contains(&(
        NodeId::Symbol {
            file: root.join("src/scoped-import-client.ts"),
            symbol: "loadTarget".to_string(),
        },
        NodeId::File(target),
        EdgeKind::DynamicImport,
    )));
}

#[cfg(unix)]
#[test]
fn scoped_symbol_aliases_skip_real_targets_outside_the_visible_namespace() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/tsconfig/symlink-workspace/link"),
    );
    let client = root.join("src/symbol-hidden-client.ts");
    let target = root.join("src/symbol-target.ts");
    let catalog = crate::codebase::ts_resolver::TsConfigCatalog::from_visible(
        &root,
        std::slice::from_ref(&root),
        &[root.join("tsconfig.json"), client.clone(), target.clone()],
    );
    let resolver = crate::codebase::ts_resolver::ScopedImportResolver::unbounded(&catalog);
    let graph_files = GraphFiles::from_files(vec![client.clone()]);
    let facts = collect_ts_facts(graph_files.indexable(), TsFactPlan::imports_and_symbols());
    let edges = collect_symbol_edges(
        &root,
        SymbolGraphFiles {
            indexable: graph_files.indexable(),
            all: graph_files.all(),
            visible: graph_files.visible(),
            graph_files: &graph_files,
        },
        &facts,
        &resolver,
        &Default::default(),
        None,
    );

    assert!(edges.iter().all(|(_, node, _)| node.as_file() != Some(target.as_path())));
}
