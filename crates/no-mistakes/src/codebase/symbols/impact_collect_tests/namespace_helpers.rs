use super::*;

#[test]
fn signature_target_symbols_preserves_chained_namespace_reexport_names() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/tests-impact-symbol/fixture"),
    );
    let target = root.join("utils.mts");
    let namespace_barrel = root.join("namespace-date-barrel.mts");
    let outer_barrel = root.join("namespace-outer-date-barrel.mts");
    let extensionless_barrel = root.join("namespace-extensionless-date-barrel.mts");
    let export_nodes = BTreeSet::from([
        NodeId::Symbol {
            file: namespace_barrel.clone(),
            symbol: "dates".to_string(),
        },
        NodeId::Symbol {
            file: outer_barrel.clone(),
            symbol: "outer".to_string(),
        },
        NodeId::Symbol {
            file: extensionless_barrel.clone(),
            symbol: "extensionlessDates".to_string(),
        },
    ]);
    let visible_files = crate::codebase::ts_source::discover_visible_paths(&root)
        .into_iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(&path))
        .collect();
    let facts = impact_test_support::signature_test_facts(&root);
    let target_symbols = signature_target_symbols(
        &target,
        "parseDate",
        &export_nodes,
        &visible_files,
        &facts,
    );

    assert_eq!(
        target_symbols.get(&namespace_barrel),
        Some(&BTreeSet::from(["dates.parseDate".to_string()]))
    );
    assert_eq!(
        target_symbols.get(&outer_barrel),
        Some(&BTreeSet::from(["outer.dates.parseDate".to_string()]))
    );
    assert_eq!(
        target_symbols.get(&extensionless_barrel),
        Some(&BTreeSet::from(["extensionlessDates.parseDate".to_string()]))
    );
}

#[test]
fn namespace_target_helpers_handle_defensive_paths() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/tests-impact-symbol/fixture"),
    );
    let invalid = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/symbols-output/fixture/src/invalid.mts"),
    );
    let namespace_barrel = root.join("namespace-date-barrel.mts");
    let mut symbols = crate::codebase::ts_symbols::FileSymbols::default();
    let visible_files = crate::codebase::ts_source::discover_visible_paths(&root)
        .into_iter()
        .map(|path| crate::codebase::ts_resolver::normalize_path(&path))
        .collect();
    let facts = impact_test_support::signature_test_facts(&root);
    assert!(!is_namespace_reexport_symbol(
        &facts,
        &root.join("missing.mts"),
        "dates"
    ));
    assert!(!is_namespace_reexport_symbol(&facts, &invalid, "dates"));
    assert!(namespace_tail_applies(
        &facts,
        &namespace_barrel,
        &symbols,
        "dates",
        "dates.parseDate",
        &visible_files,
    ));
    symbols.imports.push(crate::codebase::ts_symbols::NamedImport {
        source: "./utils.mts".to_string(),
        imported: "*".to_string(),
        local: "dates".to_string(),
        line: 1,
        is_type_only: false,
    });
    assert!(namespace_tail_applies(
        &facts,
        &namespace_barrel,
        &symbols,
        "outer",
        "dates.parseDate",
        &visible_files,
    ));
    assert!(!source_exports_symbol(
        &facts,
        Path::new("/"),
        "./missing.mts",
        "parseDate",
        &visible_files,
    ));
    assert!(!source_exports_symbol(
        &facts,
        &namespace_barrel,
        "./missing.mts",
        "parseDate",
        &visible_files,
    ));
}

#[test]
fn suggested_test_entries_ignores_file_level_edges_without_file_nodes() {
    let root = PathBuf::from("/repo");
    let graph = crate::codebase::dependencies::graph::test_support::from_typed_maps(
        root.clone(),
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
    );
    let entries = vec![NodeEntry {
        node: NodeId::Module("pkg".to_string()),
        depth: 1,
        via: vec![EdgeKind::DynamicImport],
    }];
    let suggested = suggested_test_entries(
        &graph,
        &entries,
        &[],
        &root,
        &BTreeMap::new(),
        &TsFactMap::new(),
    );
    assert_eq!(suggested, entries);
}
