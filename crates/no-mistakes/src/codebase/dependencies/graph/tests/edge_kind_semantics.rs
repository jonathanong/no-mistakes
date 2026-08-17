use super::*;
use crate::codebase::ts_source::facts::{TsFactMap, TsFileFacts};
use crate::codebase::ts_symbols::{Export, FileSymbols};

#[test]
fn workspace_paths_preserve_runtime_and_non_runtime_edge_kinds() {
    let current = p("/repo/packages/app/src/current.mts");
    let asset = p("/repo/packages/app/src/data.json");
    let target = p("/repo/packages/core/src/index.mts");
    let visible = HashSet::from([current.clone(), asset.clone(), target.clone()]);
    let graph_files = GraphFiles::from_files(visible.iter().cloned().collect());
    let tsconfig = TsConfig {
        dir: p("/repo/packages/app"),
        paths: vec![],
        paths_dir: p("/repo/packages/app"),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig).with_visible(&visible);
    let workspace = crate::codebase::workspaces::IndexedWorkspaceMap::from_packages(vec![
        crate::codebase::workspaces::WorkspacePackage {
            name: "@fixture/core".to_string(),
            dir: p("/repo/packages/core"),
            entry: Some(target.clone()),
            exports: None,
            imports: None,
        },
    ]);

    let facts = TsFactMap::from([(
        target.clone(),
        TsFileFacts {
            symbols: Some(FileSymbols {
                exports: vec![Export {
                    name: "Shape".to_string(),
                    local: None,
                    kind: ExportKind::TypeAlias,
                    line: 1,
                    is_type_only: true,
                }],
                imports: vec![],
            }),
            ..TsFileFacts::default()
        },
    )]);
    let symbols = FileSymbols::default();
    let export_inputs = ExportEdgeInputs {
        path: &current,
        symbols: &symbols,
        facts: &facts,
        resolver: &resolver,
        workspace: &workspace,
        visible_files: &visible,
        graph_files: &graph_files,
    };
    let exports = [
        ("core", "*", EdgeKind::WorkspaceImport),
        ("Shape", "Shape", EdgeKind::WorkspaceTypeImport),
    ];
    let mut edges = Vec::new();
    for (name, imported, _) in exports {
        collect_direct_reexport_edge(
            &export_inputs,
            &Export {
                name: name.to_string(),
                local: None,
                kind: ExportKind::ReExport {
                    source: "@fixture/core".to_string(),
                    imported: imported.to_string(),
                },
                line: 1,
                is_type_only: false,
            },
            name,
            &mut edges,
        );
    }
    assert_eq!(edges.len(), 2);
    assert_eq!(edges[0].2, EdgeKind::WorkspaceImport);
    assert_eq!(edges[1].2, EdgeKind::WorkspaceTypeImport);
    assert_eq!(
        edges[0].1,
        NodeId::file(target.clone()),
        "star re-exports target the workspace file"
    );
    assert_eq!(
        edges[1].1,
        NodeId::symbol(target.clone(), "Shape"),
        "named re-exports target the workspace symbol"
    );

    assert_eq!(
        import_target_with_graph_files(
            "./data.json",
            ImportKind::Type,
            &current,
            &resolver,
            &workspace,
            &visible,
            &graph_files,
        ),
        None,
        "type-only asset imports do not create runtime edges"
    );
    assert_eq!(
        import_target_with_graph_files(
            "./data.json",
            ImportKind::RequireResolve,
            &current,
            &resolver,
            &workspace,
            &visible,
            &graph_files,
        ),
        Some((NodeId::file(asset), EdgeKind::RequireResolve))
    );
    for (kind, expected) in [
        (ImportKind::Type, EdgeKind::WorkspaceTypeImport),
        (ImportKind::RequireResolve, EdgeKind::RequireResolve),
    ] {
        assert_eq!(
            import_target_with_graph_files(
                "@fixture/core",
                kind,
                &current,
                &resolver,
                &workspace,
                &visible,
                &graph_files,
            ),
            Some((NodeId::file(target.clone()), expected))
        );
    }

    let imports = [(1, ImportKind::Type), (2, ImportKind::RequireResolve)]
        .into_iter()
        .map(|(line, kind)| ExtractedImport {
            specifier: "@fixture/core".to_string(),
            kind,
            line,
            function_scope: None,
            side_effect_only: false,
            re_export: false,
            runtime_reachable: false,
        })
        .collect();
    let lazy_neighbors = import_neighbors_from_facts(
        &current,
        &TsFileFacts {
            imports,
            ..TsFileFacts::default()
        },
        &resolver,
        &workspace,
        &graph_files,
        None,
    );
    assert_eq!(
        lazy_neighbors,
        vec![
            (NodeId::file(target.clone()), EdgeKind::RequireResolve),
            (NodeId::file(target), EdgeKind::WorkspaceTypeImport),
        ]
    );
}
