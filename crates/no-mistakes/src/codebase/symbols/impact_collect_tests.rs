use super::*;
use crate::config::v2::NoMistakesConfig;

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

    assert!(target_local_names(&symbols, &file, &target_symbols, &tsconfig).is_empty());

    symbols.imports[0].is_type_only = false;
    target_symbols.insert(target, BTreeSet::new());

    assert!(target_local_names(&symbols, &file, &target_symbols, &tsconfig).is_empty());
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

#[test]
fn caller_entries_filters_export_nodes_and_non_file_nodes() {
    let root = Path::new("/repo");
    let source = PathBuf::from("/repo/src/source.mts");
    let consumer = PathBuf::from("/repo/src/consumer.mts");
    let test = PathBuf::from("/repo/src/consumer.test.mts");
    let export_node = NodeId::Symbol {
        file: source,
        symbol: "parseDate".to_string(),
    };
    let entries = vec![
        NodeEntry {
            node: export_node.clone(),
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::Module("external".to_string()),
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::Symbol {
                file: consumer,
                symbol: "format".to_string(),
            },
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::File(test),
            depth: 1,
            via: vec![EdgeKind::TestOf],
        },
    ];
    let filter = TestFileFilter::new(root, &NoMistakesConfig::default());
    let export_nodes = BTreeSet::from([export_node]);

    let production = caller_entries(&entries, root, &filter, false, &export_nodes, &[]);
    let tests = caller_entries(&entries, root, &filter, true, &export_nodes, &[]);

    assert_eq!(production.len(), 1);
    assert_eq!(production[0].file, "src/consumer.mts");
    assert_eq!(production[0].symbol.as_deref(), Some("format"));
    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0].file, "src/consumer.test.mts");
    assert_eq!(tests[0].symbol, None);
}

#[test]
fn caller_entries_merges_duplicate_callers_and_sorts() {
    let root = Path::new("/repo");
    let filter = TestFileFilter::new(root, &NoMistakesConfig::default());
    let export_nodes = BTreeSet::new();
    let entries = vec![
        NodeEntry {
            node: NodeId::Symbol {
                file: PathBuf::from("/repo/src/b.mts"),
                symbol: "beta".to_string(),
            },
            depth: 1,
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::Symbol {
                file: PathBuf::from("/repo/src/a.mts"),
                symbol: "alpha".to_string(),
            },
            depth: 1,
            via: vec![EdgeKind::Require],
        },
        NodeEntry {
            node: NodeId::Symbol {
                file: PathBuf::from("/repo/src/b.mts"),
                symbol: "beta".to_string(),
            },
            depth: 1,
            via: vec![EdgeKind::DynamicImport],
        },
    ];

    let extra = vec![CallerEntry {
        file: "src/a.mts".to_string(),
        symbol: Some("alpha".to_string()),
        depth: 2,
        via: vec!["symbol"],
    }];
    let callers = caller_entries(&entries, root, &filter, false, &export_nodes, &extra);

    assert_eq!(callers.len(), 2);
    assert_eq!(callers[0].file, "src/a.mts");
    assert_eq!(callers[0].via, vec!["require", "symbol"]);
    assert_eq!(callers[1].file, "src/b.mts");
    assert_eq!(callers[1].depth, 1);
    assert_eq!(callers[1].via, vec!["dynamic-import", "import"]);
}
