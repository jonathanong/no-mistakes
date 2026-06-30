#[test]
fn symbol_edge_helpers_cover_defensive_symbol_branches() {
    use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
    use crate::codebase::ts_symbols::{Export, FileSymbols, NamedImport};

    let current = p("/repo/src/current.mts");
    let barrel = p("/repo/src/barrel.mts");
    let mid = p("/repo/src/mid.mts");
    let source = p("/repo/src/source.mts");
    let mut visible = HashSet::new();
    visible.insert(current.clone());
    visible.insert(barrel.clone());
    visible.insert(mid.clone());
    visible.insert(source.clone());
    let tsconfig = TsConfig {
        dir: p("/repo"),
        paths: vec![],
        paths_dir: p("/repo"),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig).with_visible(&visible);
    let workspace = Default::default();
    let mut facts = TsFactMap::new();

    assert!(!target_export_is_type(&source, "missing", &facts));
    assert_eq!(
        namespace_file_node(&ImportedSymbolTarget::Node {
            node: NodeId::Module("pkg".to_string()),
            kind: EdgeKind::Import,
        }),
        (NodeId::Module("pkg".to_string()), EdgeKind::Import)
    );

    let non_reexport = Export {
        name: "plain".to_string(),
        local: None,
        kind: ExportKind::Function,
        line: 1,
        is_type_only: false,
    };
    facts.insert(
        current.clone(),
        TsFileFacts {
            symbols: Some(FileSymbols {
                exports: vec![non_reexport.clone()],
                imports: vec![],
            }),
            ..TsFileFacts::default()
        },
    );
    let current_symbols = facts
        .get_ts_facts(&current)
        .unwrap()
        .symbols
        .as_ref()
        .unwrap();
    let inputs = ExportEdgeInputs {
        path: &current,
        symbols: current_symbols,
        facts: &facts,
        resolver: &resolver,
        workspace: &workspace,
    };
    let mut candidates = Vec::new();
    let mut visited = HashSet::new();
    collect_nested_star_reexport(
        &inputs,
        &current,
        &non_reexport,
        &HashSet::new(),
        StarReexportKind {
            export_is_type_only: false,
            source_kind: EdgeKind::Import,
        },
        &mut candidates,
        &mut visited,
    );
    assert!(candidates.is_empty());

    let direct_symbols = FileSymbols {
        exports: vec![non_reexport],
        imports: vec![],
    };
    let direct_inputs = ExportEdgeInputs {
        path: &current,
        symbols: &direct_symbols,
        facts: &facts,
        resolver: &resolver,
        workspace: &workspace,
    };
    let star_self = Export {
        name: "*".to_string(),
        local: None,
        kind: ExportKind::ReExport {
            source: "./source.mts".to_string(),
            imported: "*".to_string(),
        },
        line: 2,
        is_type_only: false,
    };
    let mut edges = Vec::new();
    collect_direct_reexport_edge(&direct_inputs, &star_self, "*", &mut edges);
    assert!(edges.is_empty());

    facts.insert(
        barrel.clone(),
        TsFileFacts {
            symbols: Some(FileSymbols {
                exports: vec![Export {
                    name: "api".to_string(),
                    local: None,
                    kind: ExportKind::ReExport {
                        source: "./mid.mts".to_string(),
                        imported: "api".to_string(),
                    },
                    line: 1,
                    is_type_only: false,
                }],
                imports: vec![],
            }),
            ..TsFileFacts::default()
        },
    );
    facts.insert(
        mid.clone(),
        TsFileFacts {
            symbols: Some(FileSymbols {
                exports: vec![Export {
                    name: "api".to_string(),
                    local: None,
                    kind: ExportKind::ReExport {
                        source: "./source.mts".to_string(),
                        imported: "*".to_string(),
                    },
                    line: 1,
                    is_type_only: false,
                }],
                imports: vec![],
            }),
            ..TsFileFacts::default()
        },
    );
    assert_eq!(
        resolve_reexported_namespace_member(
            &barrel,
            "api",
            "alpha",
            EdgeKind::Import,
            &facts,
            &resolver,
            &workspace
        ),
        Some((
            NodeId::Symbol {
                file: source.clone(),
                symbol: "alpha".to_string(),
            },
            EdgeKind::Import,
        ))
    );

    let cycle = p("/repo/src/cycle.mts");
    visible.insert(cycle.clone());
    let resolver = ImportResolver::new(&tsconfig).with_visible(&visible);
    facts.insert(
        cycle.clone(),
        TsFileFacts {
            symbols: Some(FileSymbols {
                exports: vec![
                    Export {
                        name: "other".to_string(),
                        local: None,
                        kind: ExportKind::Function,
                        line: 1,
                        is_type_only: false,
                    },
                    Export {
                        name: "api".to_string(),
                        local: None,
                        kind: ExportKind::ReExport {
                            source: "./cycle.mts".to_string(),
                            imported: "api".to_string(),
                        },
                        line: 2,
                        is_type_only: false,
                    },
                ],
                imports: vec![NamedImport {
                    source: "./missing.mts".to_string(),
                    imported: "*".to_string(),
                    local: "api".to_string(),
                    line: 3,
                    is_type_only: false,
                }],
            }),
            ..TsFileFacts::default()
        },
    );
    assert_eq!(
        resolve_reexported_namespace_member(
            &cycle,
            "api",
            "alpha",
            EdgeKind::Import,
            &facts,
            &resolver,
            &workspace
        ),
        None
    );

    let prefix_only_options = GraphConfigOptions {
        route: crate::codebase::config::RouteOptions::default(),
        queue: crate::codebase::config::QueueOptions::default(),
        http_route: crate::codebase::config::HttpRouteOptions::default(),
        http_call: crate::codebase::config::HttpCallOptions {
            backend_prefixes: vec!["/api/".to_string()],
        },
        project_route_globset: None,
        test_filter: None,
        rewrites: vec![],
        queue_project_factory_names: vec![],
        dotnet_projects: vec![],
        swift_packages: vec![],
        terraform: Default::default(),
    };
    assert!(collect_symbol_http_route_defs(
        Path::new("/repo"),
        &[],
        &facts,
        Some(&prefix_only_options),
    )
    .is_empty());
    assert!(symbol_http_targets(
        &current,
        &[FunctionCall {
            caller: Some("api".to_string()),
            callee: "fetch".to_string(),
            static_arg: None,
            static_cwd: None,
        }],
        &[(source.clone(), "/api/:id".to_string())],
    )
    .is_empty());
}
