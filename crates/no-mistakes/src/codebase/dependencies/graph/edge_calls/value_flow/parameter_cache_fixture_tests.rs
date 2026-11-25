use std::collections::BTreeSet;
use std::path::PathBuf;

#[test]
fn saved_stage_workload_preserves_all_48_exact_findings() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/query-reached-per-item/stage-import-index-workload");
    let fixture = crate::test_support::materialize_saved_fixture(&source);
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .unwrap();
    let findings = pool
        .install(|| crate::codebase::rules::run_check(fixture.path(), None, None))
        .unwrap();
    let actual = findings
        .iter()
        .map(|finding| {
            (
                finding.file.as_str(),
                finding.line,
                finding.import.as_deref().unwrap_or_default(),
            )
        })
        .collect::<BTreeSet<_>>();
    let expected = (0..24)
        .flat_map(|stage| {
            [
                ("src/caller.mts", 28 + stage * 4, "stage.run"),
                ("src/caller.mts", 29 + stage * 4, "stage.check"),
            ]
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
    assert_eq!(findings.len(), 48);
}
