use super::*;
use crate::config::v2::NoMistakesConfig;

#[test]
fn caller_parts_ignores_non_file_backed_nodes() {
    let root = Path::new("/repo");

    assert!(caller_parts(&NodeId::Module("react".to_string()), root).is_none());
    assert!(
        caller_parts(
            &NodeId::QueueJob {
                queue_file: PathBuf::from("/repo/queue.mts"),
                job: "send-email".to_string(),
            },
            root,
        )
        .is_none()
    );
}

#[test]
fn export_location_errors_include_file_context() {
    let root = Path::new("/repo");
    let missing = root.join("missing.mts");
    let missing_err = export_location(&missing, root, "parseDate", false).unwrap_err();
    assert!(format!("{missing_err:#}").contains("reading /repo/missing.mts"));

    let invalid = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/codebase-analysis/symbols-output/fixture/src/invalid.mts");
    let parse_err = export_location(&invalid, root, "parseDate", false).unwrap_err();
    assert!(format!("{parse_err:#}").contains("extracting symbols from"));
}

#[test]
fn suggested_tests_merges_duplicate_test_files() {
    let root = Path::new("/repo");
    let filter = TestFileFilter::new(root, &NoMistakesConfig::default());
    let test_file = PathBuf::from("/repo/src/date.test.mts");
    let entries = vec![
        NodeEntry {
            node: NodeId::File(test_file.clone()),
            depth: 3,
            via: vec![EdgeKind::Import],
        },
        NodeEntry {
            node: NodeId::Symbol {
                file: test_file,
                symbol: "coversDate".to_string(),
            },
            depth: 1,
            via: vec![EdgeKind::TestOf],
        },
        NodeEntry {
            node: NodeId::Module("vitest".to_string()),
            depth: 1,
            via: vec![EdgeKind::Import],
        },
    ];

    let tests = suggested_tests(&entries, root, &filter);

    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0].file, "src/date.test.mts");
    assert_eq!(tests[0].depth, 1);
    assert_eq!(tests[0].via, vec!["import", "test"]);
}
