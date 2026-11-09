thread_local! {
    pub(super) static PREFIX_SCANS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

macro_rules! record_annotation_prefix_scan {
    () => {
        $crate::integration_tests::calls::tests::PREFIX_SCANS
            .with(|count| count.set(count.get() + 1));
    };
}

use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/integration-tests/annotation-adjacency/fixture")
}

#[derive(serde::Deserialize)]
struct Case {
    name: String,
    source: String,
    expected: Option<String>,
    scans: usize,
    empty: bool,
}

#[test]
fn annotation_adjacency_preserves_directive_results_and_counts_real_prefix_scans() {
    let cases: Vec<Case> =
        serde_json::from_str(&std::fs::read_to_string(fixture().join("cases.json")).unwrap())
            .unwrap();
    for case in cases {
        PREFIX_SCANS.with(|count| count.set(0));
        let start = if case.empty {
            0
        } else {
            case.source.find("function target").unwrap() as u32
        };
        let actual =
            super::integration_annotation_before(&case.source, oxc_span::Span::new(start, start));
        assert_eq!(actual, case.expected, "{}", case.name);
        assert_eq!(
            PREFIX_SCANS.with(std::cell::Cell::get),
            case.scans,
            "{}",
            case.name
        );
    }
}

#[test]
fn comment_free_helpers_do_not_reverse_scan_their_preceding_source() {
    let path = fixture().join("comment-free.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    PREFIX_SCANS.with(|count| count.set(0));
    let analysis = crate::ast::with_program(&path, &source, |program, source| {
        crate::integration_tests::analysis::analyze_program(&path, program, source)
    })
    .unwrap();
    assert_eq!(analysis.functions.len(), 3);
    assert_eq!(PREFIX_SCANS.with(std::cell::Cell::get), 0);
}
