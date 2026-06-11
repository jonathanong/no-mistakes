use super::*;
#[test]
fn exported_symbol_for_local_skips_re_exports_and_type_exports() {
    let symbols = crate::codebase::ts_symbols::FileSymbols {
        exports: vec![
            crate::codebase::ts_symbols::Export {
                name: "parseDate".to_string(),
                local: None,
                kind: ExportKind::ReExport {
                    source: "./utils.mts".to_string(),
                    imported: "parseDate".to_string(),
                },
                line: 1,
                is_type_only: false,
            },
            crate::codebase::ts_symbols::Export {
                name: "DateLike".to_string(),
                local: None,
                kind: ExportKind::TypeAlias,
                line: 2,
                is_type_only: true,
            },
            crate::codebase::ts_symbols::Export {
                name: "formatDate".to_string(),
                local: Some("format".to_string()),
                kind: ExportKind::Const,
                line: 3,
                is_type_only: false,
            },
        ],
        imports: vec![],
    };

    assert_eq!(
        exported_symbol_for_local(&symbols, "format").as_deref(),
        Some("formatDate")
    );
    assert_eq!(exported_symbol_for_local(&symbols, "parseDate"), None);
    assert_eq!(exported_symbol_for_local(&symbols, "DateLike"), None);
}

#[test]
fn target_local_names_skips_type_only_imports_and_empty_export_sets() {
    let root = crate::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-cases/codebase-analysis/tests-impact-symbol/fixture"),
    );
    let file = root.join("private-caller-with-export.mts");
    let target = root.join("utils.mts");
    let tsconfig = crate::codebase::ts_resolver::TsConfig {
        dir: root.clone(),
        paths: vec![],
        paths_dir: root,
        base_url: None,
    };
    let mut symbols = crate::codebase::ts_symbols::FileSymbols {
        exports: vec![],
        imports: vec![crate::codebase::ts_symbols::NamedImport {
            source: "./utils.mts".to_string(),
            imported: "parseDate".to_string(),
            local: "parseDate".to_string(),
            line: 1,
            is_type_only: true,
        }],
    };
    let mut target_symbols = BTreeMap::from([(target.clone(), BTreeSet::from(["parseDate".to_string()]))]);
    let workspace = crate::codebase::workspaces::WorkspaceMap::default();

    assert!(target_local_names(&symbols, &file, &target_symbols, &tsconfig, &workspace).is_empty());

    symbols.imports[0].is_type_only = false;
    target_symbols.insert(target, BTreeSet::new());

    assert!(target_local_names(&symbols, &file, &target_symbols, &tsconfig, &workspace).is_empty());
}

#[test]
fn signature_target_symbols_keeps_file_entries_and_ignores_non_file_nodes() {
    let target = PathBuf::from("/repo/src/utils.mts");
    let barrel = PathBuf::from("/repo/src/barrel.mts");
    let queue = PathBuf::from("/repo/src/queue.mts");
    let export_nodes = BTreeSet::from([
        NodeId::File(barrel.clone()),
        NodeId::Module("external".to_string()),
        NodeId::QueueJob {
            queue_file: queue,
            job: "send".to_string(),
        },
    ]);

    let target_symbols = signature_target_symbols(&target, "parseDate", &export_nodes);

    assert_eq!(
        target_symbols.get(&target),
        Some(&BTreeSet::from(["parseDate".to_string()]))
    );
    assert_eq!(target_symbols.get(&barrel), Some(&BTreeSet::new()));
    assert_eq!(target_symbols.len(), 2);
}
