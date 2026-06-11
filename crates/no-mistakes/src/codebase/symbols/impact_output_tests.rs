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
fn export_name_normalizes_default_exports() {
    assert_eq!(export_name(&ExportKind::Default, "NamedDefault"), "default");
    assert_eq!(export_name(&ExportKind::Function, "parseDate"), "parseDate");
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

    let extra = vec![CallerEntry {
        file: "src/date.test.mts".to_string(),
        symbol: None,
        depth: 2,
        via: vec!["symbol"],
    }];

    let tests = suggested_tests(&entries, root, &filter, &extra);

    assert_eq!(tests.len(), 1);
    assert_eq!(tests[0].file, "src/date.test.mts");
    assert_eq!(tests[0].depth, 1);
    assert_eq!(tests[0].via, vec!["import", "symbol", "test"]);
}

#[test]
fn markdown_report_uses_symbol_title_when_roots_are_empty() {
    let report = SignatureImpactReport {
        roots: vec![],
        symbol: "parseDate".to_string(),
        definition: SymbolLocation {
            file: "src/date.mts".to_string(),
            symbol: "parseDate".to_string(),
            line: 1,
            kind: "const",
        },
        exports: vec![],
        production_callers: vec![],
        test_callers: vec![],
        suggested_tests: vec![],
        warnings: vec![],
    };
    let mut out = Vec::new();

    write_report(&report, Format::Md, &mut out).unwrap();

    let rendered = String::from_utf8(out).unwrap();
    assert!(rendered.starts_with("# `parseDate`"));
}
