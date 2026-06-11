use super::*;
use crate::config::v2::NoMistakesConfig;

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
    let context = CallerEntriesContext {
        root,
        test_filter: &filter,
        export_nodes: &export_nodes,
        target_symbol: "parseDate",
    };

    let production = caller_entries(&entries, &context, false, &[]);
    let tests = caller_entries(&entries, &context, true, &[]);

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
    let context = CallerEntriesContext {
        root,
        test_filter: &filter,
        export_nodes: &export_nodes,
        target_symbol: "beta",
    };
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
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::Symbol {
                file: PathBuf::from("/repo/src/b.mts"),
                symbol: "beta".to_string(),
            },
            depth: 1,
            via: vec![EdgeKind::Import],
        },
    ];

    let extra = vec![CallerEntry {
        file: "src/a.mts".to_string(),
        symbol: Some("alpha".to_string()),
        depth: 2,
        via: vec!["symbol"],
    }];
    let callers = caller_entries(&entries, &context, false, &extra);

    assert_eq!(callers.len(), 2);
    assert_eq!(callers[0].file, "src/a.mts");
    assert_eq!(callers[0].via, vec!["import", "symbol"]);
    assert_eq!(callers[1].file, "src/b.mts");
    assert_eq!(callers[1].depth, 1);
    assert_eq!(callers[1].via, vec!["import"]);
}

#[test]
fn file_entry_uses_symbol_checks_extracted_and_alias_member_uses() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/tests-impact-symbol/fixture");

    assert!(file_entry_uses_symbol(
        &root,
        "require-caller.mts",
        "parseDate"
    ));
    assert!(file_entry_uses_symbol(
        &root,
        "dynamic-import-caller.mts",
        "parseDate"
    ));
    assert!(file_entry_uses_symbol(
        &root,
        "dynamic-import-alias-caller.mts",
        "parseDate"
    ));
    assert!(!file_entry_uses_symbol(
        &root,
        "dynamic-import-unused.mts",
        "parseDate"
    ));
    assert!(!file_entry_uses_symbol(
        &root,
        "missing-dynamic-import-caller.mts",
        "parseDate"
    ));
}

#[test]
fn symbol_aliases_collect_destructured_and_member_assignment_locals() {
    let aliases = symbol_aliases_in_source(
        "const { parseDate: pd } = await import('./utils.mts');\n\
         const readDate = require('./utils.mts').parseDate;\n\
         return utils.parseDate;\n\
         assigned = utils.parseDate;\n\
         pd(value); readDate(value);",
        "parseDate",
    );

    assert!(aliases.contains("pd"));
    assert!(aliases.contains("readDate"));
}

#[test]
fn local_callee_matching_accepts_namespace_members() {
    assert!(matches_local_callee(
        "dates.parseDate",
        &BTreeSet::from(["dates".to_string()])
    ));
    assert!(matches_local_callee(
        "parseDate",
        &BTreeSet::from(["parseDate".to_string()])
    ));
    assert!(!matches_local_callee(
        "updatedDates.parseDate",
        &BTreeSet::from(["dates".to_string()])
    ));
}
