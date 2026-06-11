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

    let production = caller_entries(&entries, root, &filter, false, &export_nodes);
    let tests = caller_entries(&entries, root, &filter, true, &export_nodes);

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

    let callers = caller_entries(&entries, root, &filter, false, &export_nodes);

    assert_eq!(callers.len(), 2);
    assert_eq!(callers[0].file, "src/a.mts");
    assert_eq!(callers[0].via, vec!["require"]);
    assert_eq!(callers[1].file, "src/b.mts");
    assert_eq!(callers[1].depth, 1);
    assert_eq!(callers[1].via, vec!["dynamic-import", "import"]);
}
