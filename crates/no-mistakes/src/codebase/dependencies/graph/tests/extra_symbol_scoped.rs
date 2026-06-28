#[test]
fn scoped_import_targets_preserve_workspace_edges() {
    let current = p("/repo/packages/app/src/current.mts");
    let target = p("/repo/packages/core/src/index.mts");
    let tsconfig = TsConfig {
        dir: p("/repo"),
        paths: vec![],
        paths_dir: p("/repo"),
        base_url: None,
    };
    let resolver = ImportResolver::new(&tsconfig);
    let workspace = crate::codebase::workspaces::WorkspaceMap {
        packages: vec![crate::codebase::workspaces::WorkspacePackage {
            name: "@fixture/core".to_string(),
            dir: p("/repo/packages/core"),
            entry: Some(target.clone()),
            exports: None,
            imports: None,
        }],
    };

    assert_eq!(
        import_target(
            "@fixture/core",
            ImportKind::Dynamic,
            &current,
            &resolver,
            &workspace,
        ),
        Some((NodeId::File(target.clone()), EdgeKind::WorkspaceImport)),
    );

    let scoped = scoped_import_map(
        &[ExtractedImport {
            specifier: "@fixture/core".to_string(),
            kind: ImportKind::Dynamic,
        line: 1,
            function_scope: Some("run".to_string()),
        side_effect_only: false,
        re_export: false,
        runtime_reachable: false,
        }],
        &current,
        &resolver,
        &workspace,
    );

    assert_eq!(
        scoped.get("run"),
        Some(&vec![(NodeId::File(target), EdgeKind::WorkspaceImport)]),
    );
}

#[test]
fn symbol_fallback_imports_keep_only_top_level_uses_when_exports_exist() {
    use crate::codebase::dependencies::extract::FunctionCall;

    let alpha = ImportedSymbolTarget::Symbol {
        file: p("/repo/src/source.mts"),
        symbol: "alpha".to_string(),
        kind: EdgeKind::Import,
    };
    let beta = ImportedSymbolTarget::Symbol {
        file: p("/repo/src/source.mts"),
        symbol: "beta".to_string(),
        kind: EdgeKind::Import,
    };
    let mut imports = HashMap::new();
    imports.insert("alpha".to_string(), alpha.clone());
    imports.insert("beta".to_string(), beta);
    let calls = vec![
        FunctionCall {
            caller: None,
            callee: "alpha".to_string(),
            static_arg: None,
            static_cwd: None,
        },
        FunctionCall {
            caller: None,
            callee: "beta".to_string(),
            static_arg: None,
            static_cwd: None,
        },
        FunctionCall {
            caller: None,
            callee: "alpha".to_string(),
            static_arg: None,
            static_cwd: None,
        },
        FunctionCall {
            caller: None,
            callee: "missing".to_string(),
            static_arg: None,
            static_cwd: None,
        },
        FunctionCall {
            caller: Some("run".to_string()),
            callee: "beta".to_string(),
            static_arg: None,
            static_cwd: None,
        },
    ];

    let selected = fallback_imported_symbols(false, &calls, &[], &imports);
    assert_eq!(selected.len(), 2);
    assert!(selected
        .iter()
        .any(|target| target_node(target) == target_node(&alpha)));

    imports.insert("alpha_alias".to_string(), alpha.clone());
    let all = fallback_imported_symbols(true, &[], &[], &imports);
    assert_eq!(all.len(), 2);
}
