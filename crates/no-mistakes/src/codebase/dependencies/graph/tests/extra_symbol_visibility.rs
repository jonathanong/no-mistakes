#[test]
fn symbol_edges_reject_workspace_targets_outside_visible_files() {
    use crate::codebase::dependencies::extract::{ExtractedImport, ImportKind};
    use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
    use crate::codebase::ts_symbols::{Export, FileSymbols, NamedImport};

    let current = p("/repo/packages/app/src/current.mts");
    let hidden_target = p("/repo/packages/core/dist/index.mts");
    let visible = HashSet::from([current.clone()]);
    let tsconfig = TsConfig {
        dir: p("/repo/packages/app"),
        paths: vec![],
        paths_dir: p("/repo/packages/app"),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig).with_visible(&visible);
    let workspace = crate::codebase::workspaces::IndexedWorkspaceMap::from_packages(vec![
        crate::codebase::workspaces::WorkspacePackage {
            name: "@fixture/hidden".to_string(),
            dir: p("/repo/packages/core"),
            entry: Some(hidden_target.clone()),
            exports: None,
            imports: None,
        },
    ]);
    let symbols = FileSymbols {
        exports: vec![
            Export {
                name: "hiddenNamed".to_string(),
                local: None,
                kind: ExportKind::ReExport {
                    source: "@fixture/hidden".to_string(),
                    imported: "hiddenNamed".to_string(),
                },
                line: 1,
                is_type_only: false,
            },
            Export {
                name: "*".to_string(),
                local: None,
                kind: ExportKind::ReExport {
                    source: "@fixture/hidden".to_string(),
                    imported: "*".to_string(),
                },
                line: 2,
                is_type_only: false,
            },
            Export {
                name: "hiddenNs".to_string(),
                local: None,
                kind: ExportKind::ReExport {
                    source: "@fixture/hidden".to_string(),
                    imported: "*".to_string(),
                },
                line: 3,
                is_type_only: false,
            },
        ],
        imports: vec![
            NamedImport {
                source: "@fixture/hidden".to_string(),
                imported: "hiddenNamed".to_string(),
                local: "hiddenNamed".to_string(),
                line: 4,
                is_type_only: false,
            },
            NamedImport {
                source: "@fixture/hidden".to_string(),
                imported: "*".to_string(),
                local: "hiddenNs".to_string(),
                line: 5,
                is_type_only: false,
            },
        ],
    };
    let mut facts = TsFactMap::new();
    facts.insert(
        current.clone(),
        TsFileFacts {
            symbols: Some(symbols),
            imports: vec![ExtractedImport {
                specifier: "@fixture/hidden".to_string(),
                kind: ImportKind::Dynamic,
                line: 6,
                function_scope: Some("hiddenNamed".to_string()),
                side_effect_only: false,
                re_export: false,
                runtime_reachable: true,
            }],
            ..TsFileFacts::default()
        },
    );
    facts.insert(
        hidden_target.clone(),
        TsFileFacts {
            symbols: Some(FileSymbols {
                exports: vec![Export {
                    name: "hiddenNamed".to_string(),
                    local: None,
                    kind: ExportKind::Function,
                    line: 1,
                    is_type_only: false,
                }],
                imports: vec![],
            }),
            ..TsFileFacts::default()
        },
    );

    let edges = collect_symbol_edges(
        Path::new("/repo"),
        SymbolGraphFiles {
            indexable: std::slice::from_ref(&current),
            all: std::slice::from_ref(&current),
            visible: &visible,
            graph_files: &GraphFiles::from_files(visible.iter().cloned().collect()),
        },
        &facts,
        &resolver,
        &workspace,
        None,
    );

    assert!(!edges.iter().any(|(from, to, _)| {
        from.as_file() == Some(hidden_target.as_path())
            || to.as_file() == Some(hidden_target.as_path())
    }));
    assert!(!edges
        .iter()
        .any(|(_, to, _)| { *to == NodeId::Module("@fixture/hidden".to_string()) }));
    assert_eq!(
        import_target(
            "@fixture/hidden",
            ImportKind::Dynamic,
            &current,
            &resolver,
            &workspace,
            &visible,
        ),
        None
    );
    assert_eq!(
        resolve_reexported_namespace_member(
            &current,
            "hiddenNs",
            "value",
            EdgeKind::Import,
            ReexportNamespaceInputs {
                facts: &facts,
                resolver: &resolver,
                workspace: &workspace,
                visible_files: &visible,
                graph_files: &GraphFiles::from_files(visible.iter().cloned().collect()),
            },
        ),
        None
    );
}
