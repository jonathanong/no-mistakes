#[test]
fn symbol_edge_helpers_cover_defensive_and_workspace_paths() {
    use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
    use crate::codebase::ts_symbols::{Export, FileSymbols, NamedImport};

    let current = p("/repo/packages/app/src/current.mts");
    let asset = p("/repo/packages/app/src/data.json");
    let barrel = p("/repo/packages/app/src/barrel.mts");
    let workspace_target = p("/repo/packages/core/src/index.mts");
    let mut visible = HashSet::new();
    visible.insert(current.clone());
    visible.insert(asset.clone());
    visible.insert(barrel.clone());
    let tsconfig = TsConfig {
        dir: p("/repo/packages/app"),
        paths: vec![],
        paths_dir: p("/repo/packages/app"),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig).with_visible(&visible);
    let workspace = crate::codebase::workspaces::WorkspaceMap {
        packages: vec![crate::codebase::workspaces::WorkspacePackage {
            name: "@fixture/core".to_string(),
            dir: p("/repo/packages/core"),
            entry: Some(workspace_target.clone()),
            exports: None,
            imports: None,
        }],
    };

    let symbols = FileSymbols {
        exports: vec![],
        imports: vec![
            NamedImport {
                source: "@fixture/core".to_string(),
                imported: "workspaceValue".to_string(),
                local: "workspaceValue".to_string(),
                line: 1,
                is_type_only: false,
            },
            NamedImport {
                source: "./data.json".to_string(),
                imported: "payload".to_string(),
                local: "payload".to_string(),
                line: 2,
                is_type_only: false,
            },
            NamedImport {
                source: "react".to_string(),
                imported: "useMemo".to_string(),
                local: "useMemo".to_string(),
                line: 3,
                is_type_only: false,
            },
            NamedImport {
                source: "./missing.mts".to_string(),
                imported: "missing".to_string(),
                local: "missing".to_string(),
                line: 4,
                is_type_only: false,
            },
            NamedImport {
                source: "@fixture/core".to_string(),
                imported: "*".to_string(),
                local: "core".to_string(),
                line: 5,
                is_type_only: true,
            },
            NamedImport {
                source: "zod".to_string(),
                imported: "*".to_string(),
                local: "z".to_string(),
                line: 6,
                is_type_only: false,
            },
            NamedImport {
                source: "./nope.mts".to_string(),
                imported: "*".to_string(),
                local: "nope".to_string(),
                line: 7,
                is_type_only: false,
            },
        ],
    };

    let imported = imported_symbol_map(&current, &symbols, &resolver, &workspace);
    assert_eq!(
        target_node(imported.get("workspaceValue").unwrap()),
        (
            NodeId::Symbol {
                file: workspace_target.clone(),
                symbol: "workspaceValue".to_string()
            },
            EdgeKind::WorkspaceImport
        )
    );
    assert_eq!(
        target_node(imported.get("payload").unwrap()),
        (NodeId::File(asset), EdgeKind::AssetImport)
    );
    assert_eq!(
        target_node(imported.get("useMemo").unwrap()),
        (NodeId::Module("react".to_string()), EdgeKind::Import)
    );
    assert!(!imported.contains_key("missing"));

    let namespaces = namespace_import_map(&current, &symbols, &resolver, &workspace);
    assert_eq!(
        namespace_target_node(namespaces.get("core").unwrap(), "parse"),
        (
            NodeId::Symbol {
                file: workspace_target.clone(),
                symbol: "parse".to_string()
            },
            EdgeKind::WorkspaceImport
        )
    );
    assert_eq!(
        namespace_target_node(namespaces.get("z").unwrap(), "object"),
        (NodeId::Module("zod".to_string()), EdgeKind::Import)
    );
    assert!(!namespaces.contains_key("nope"));

    let mut facts = TsFactMap::new();
    facts.insert(
        barrel.clone(),
        TsFileFacts {
            symbols: Some(FileSymbols {
                exports: vec![
                    Export {
                        name: "ignored".to_string(),
                        local: None,
                        kind: ExportKind::Function,
                        line: 1,
                        is_type_only: false,
                    },
                    Export {
                        name: "workspaceValue".to_string(),
                        local: None,
                        kind: ExportKind::ReExport {
                            source: "@fixture/core".to_string(),
                            imported: "*".to_string(),
                        },
                        line: 2,
                        is_type_only: true,
                    },
                ],
                imports: vec![],
            }),
            ..TsFileFacts::default()
        },
    );
    let mut imported_barrel = HashMap::new();
    imported_barrel.insert(
        "barrel".to_string(),
        ImportedSymbolTarget::Symbol {
            file: barrel,
            symbol: "workspaceValue".to_string(),
            kind: EdgeKind::Import,
        },
    );
    assert_eq!(
        resolve_imported_callee(
            "barrel.member",
            &imported_barrel,
            &HashMap::new(),
            &facts,
            &resolver,
            &workspace
        ),
        Some((
            NodeId::Symbol {
                file: workspace_target,
                symbol: "member".to_string()
            },
            EdgeKind::TypeImport
        ))
    );
    assert!(resolve_imported_callee(
        "barrel",
        &imported_barrel,
        &HashMap::new(),
        &facts,
        &resolver,
        &workspace
    )
    .is_some());
    assert!(resolve_imported_callee(
        "barrel.member",
        &HashMap::new(),
        &HashMap::new(),
        &facts,
        &resolver,
        &workspace
    )
    .is_none());
}

#[test]
fn star_reexport_edges_skip_invalid_default_and_unresolved_targets() {
    use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
    use crate::codebase::ts_symbols::{Export, FileSymbols};

    let current = p("/repo/src/current.mts");
    let target = p("/repo/src/target.mts");
    let mut visible = HashSet::new();
    visible.insert(current.clone());
    visible.insert(target.clone());
    let tsconfig = TsConfig {
        dir: p("/repo"),
        paths: vec![],
        paths_dir: p("/repo"),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig).with_visible(&visible);
    let symbols = FileSymbols {
        exports: vec![
            Export {
                name: "*".to_string(),
                local: None,
                kind: ExportKind::ReExport {
                    source: "./missing.mts".to_string(),
                    imported: "*".to_string(),
                },
                line: 1,
                is_type_only: false,
            },
            Export {
                name: "notStar".to_string(),
                local: None,
                kind: ExportKind::ReExport {
                    source: "./target.mts".to_string(),
                    imported: "named".to_string(),
                },
                line: 2,
                is_type_only: false,
            },
            Export {
                name: "*".to_string(),
                local: None,
                kind: ExportKind::ReExport {
                    source: "./target.mts".to_string(),
                    imported: "*".to_string(),
                },
                line: 3,
                is_type_only: false,
            },
        ],
        imports: vec![],
    };
    let mut facts = TsFactMap::new();
    facts.insert(
        current.clone(),
        TsFileFacts {
            symbols: Some(symbols),
            ..TsFileFacts::default()
        },
    );
    facts.insert(
        target.clone(),
        TsFileFacts {
            symbols: Some(FileSymbols {
                exports: vec![
                    Export {
                        name: "default".to_string(),
                        local: None,
                        kind: ExportKind::Default,
                        line: 1,
                        is_type_only: false,
                    },
                    Export {
                        name: "*".to_string(),
                        local: None,
                        kind: ExportKind::ReExport {
                            source: "./other.mts".to_string(),
                            imported: "*".to_string(),
                        },
                        line: 2,
                        is_type_only: false,
                    },
                    Export {
                        name: "keep".to_string(),
                        local: None,
                        kind: ExportKind::Function,
                        line: 3,
                        is_type_only: false,
                    },
                ],
                imports: vec![],
            }),
            ..TsFileFacts::default()
        },
    );

    let edges = collect_symbol_edges(
        std::slice::from_ref(&current),
        &facts,
        &resolver,
        &Default::default(),
    );

    assert!(edges.contains(&(
        NodeId::Symbol {
            file: current.clone(),
            symbol: "keep".to_string(),
        },
        NodeId::Symbol {
            file: target,
            symbol: "keep".to_string(),
        },
        EdgeKind::Import
    )));
    assert!(!edges.iter().any(|(from, _, _)| {
        *from
            == NodeId::Symbol {
                file: current.clone(),
                symbol: "default".to_string(),
            }
    }));
}

#[test]
fn symbol_edge_helpers_cover_unreachable_export_and_barrel_fallback_paths() {
    use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
    use crate::codebase::ts_symbols::{Export, FileSymbols};

    let current = p("/repo/src/current.mts");
    let barrel = p("/repo/src/barrel.mts");
    let mut visible = HashSet::new();
    visible.insert(current.clone());
    visible.insert(barrel.clone());
    let tsconfig = TsConfig {
        dir: p("/repo"),
        paths: vec![],
        paths_dir: p("/repo"),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig).with_visible(&visible);
    let workspace = Default::default();
    let mut facts = TsFactMap::new();
    facts.insert(
        barrel.clone(),
        TsFileFacts {
            symbols: Some(FileSymbols {
                exports: vec![
                    Export {
                        name: "barrelValue".to_string(),
                        local: None,
                        kind: ExportKind::Function,
                        line: 1,
                        is_type_only: false,
                    },
                    Export {
                        name: "barrelValue".to_string(),
                        local: None,
                        kind: ExportKind::ReExport {
                            source: "./target.mts".to_string(),
                            imported: "named".to_string(),
                        },
                        line: 2,
                        is_type_only: false,
                    },
                ],
                imports: vec![],
            }),
            ..TsFileFacts::default()
        },
    );
    let inputs = ExportEdgeInputs {
        path: &current,
        symbols: facts.get_ts_facts(&barrel).unwrap().symbols.as_ref().unwrap(),
        facts: &facts,
        resolver: &resolver,
        workspace: &workspace,
    };
    let mut edges = Vec::new();
    collect_star_reexport_edges(
        &inputs,
        &Export {
            name: "plain".to_string(),
            local: None,
            kind: ExportKind::Function,
            line: 1,
            is_type_only: false,
        },
        &mut edges,
    );
    collect_star_reexport_edges(
        &inputs,
        &Export {
            name: "*".to_string(),
            local: None,
            kind: ExportKind::ReExport {
                source: "./target.mts".to_string(),
                imported: "named".to_string(),
            },
            line: 2,
            is_type_only: false,
        },
        &mut edges,
    );
    assert!(edges.is_empty());

    let mut imported = HashMap::new();
    imported.insert(
        "barrel".to_string(),
        ImportedSymbolTarget::Symbol {
            file: barrel,
            symbol: "barrelValue".to_string(),
            kind: EdgeKind::Import,
        },
    );
    assert_eq!(
        resolve_imported_callee(
            "barrel.member",
            &imported,
            &HashMap::new(),
            &facts,
            &resolver,
            &workspace
        ),
        None
    );
}

#[test]
fn symbol_bfs_records_alternate_via_kinds_for_existing_nodes() {
    let root = NodeId::Symbol {
        file: p("/repo/src/root.mts"),
        symbol: "root".to_string(),
    };
    let left = NodeId::Symbol {
        file: p("/repo/src/left.mts"),
        symbol: "left".to_string(),
    };
    let right = NodeId::Symbol {
        file: p("/repo/src/right.mts"),
        symbol: "right".to_string(),
    };
    let target = NodeId::Module("react".to_string());
    let mut edges = EdgeMap::new();
    edges.insert(
        root.clone(),
        vec![(left.clone(), EdgeKind::Import), (right.clone(), EdgeKind::Import)],
    );
    edges.insert(left, vec![(target.clone(), EdgeKind::Import)]);
    edges.insert(right, vec![(target.clone(), EdgeKind::DynamicImport)]);

    let entries = bfs_skipping_symbol_owner_files(&[root], &edges, None, None);
    let target_entry = entries
        .iter()
        .find(|entry| entry.node == target)
        .expect("target module should be reached once");

    assert_eq!(
        target_entry.via,
        vec![EdgeKind::Import, EdgeKind::DynamicImport]
    );
}
