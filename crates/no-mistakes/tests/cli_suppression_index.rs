use super::check_json_snapshot::assert_saved_fixture_report;
use std::path::PathBuf;

#[test]
fn suppression_index_compatibility_check_json_snapshot() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/forbidden-calls/suppression-index/fixture");
    let cases = [(
        "mixed_directives",
        root.clone(),
        root.join(".no-mistakes.yml"),
    )];
    let reports = assert_saved_fixture_report(
        "suppression_index_compatibility",
        &cases,
        &[],
        &["--include-suppressed"],
    );
    let report = &reports["mixed_directives"]["report"];
    let reported: Vec<_> = report["rules"]
        .as_array()
        .unwrap()
        .iter()
        .map(|finding| finding["line"].as_u64().unwrap())
        .collect();
    assert_eq!(reported, [2, 7]);
    let suppressed = report["suppressed"].as_array().unwrap();
    assert_eq!(suppressed.len(), 3);
    for (file, line, kind, directive_line) in [
        ("file.ts", 3, "file", 1),
        ("worker.ts", 4, "nextLine", 3),
        ("worker.ts", 5, "line", 5),
    ] {
        assert!(suppressed.iter().any(|finding| {
            finding["file"] == file
                && finding["line"] == line
                && finding["directive"]["kind"] == kind
                && finding["directive"]["line"] == directive_line
        }));
    }
}
