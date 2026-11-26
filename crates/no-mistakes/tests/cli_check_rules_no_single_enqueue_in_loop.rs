use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/no-single-enqueue-in-loop/fixtures")
        .join(name)
}

#[test]
fn no_single_enqueue_in_loop_check_json_snapshot() {
    let configured = fixture("configured");
    let empty = fixture("empty-targets");
    let same_export = fixture("same-export");
    let sibling_scopes = fixture("sibling-scopes");
    let spellings = fixture("spellings");
    let suppressions = fixture("suppressions");
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "no_single_enqueue_in_loop",
        &[
            (
                "configured",
                configured.clone(),
                configured.join(".no-mistakes.yml"),
            ),
            ("empty", empty.clone(), empty.join(".no-mistakes.yml")),
            (
                "same-export",
                same_export.clone(),
                same_export.join(".no-mistakes.yml"),
            ),
            (
                "sibling-scopes",
                sibling_scopes.clone(),
                sibling_scopes.join(".no-mistakes.yml"),
            ),
            (
                "spellings",
                spellings.clone(),
                spellings.join(".no-mistakes.yml"),
            ),
            (
                "suppressions",
                suppressions.clone(),
                suppressions.join(".no-mistakes.yml"),
            ),
        ],
        &[],
        &[],
    );
    assert_eq!(reports["configured"]["exitCode"], 1);
    assert_eq!(reports["same-export"]["exitCode"], 1);
    assert_eq!(reports["sibling-scopes"]["exitCode"], 1);
    assert_eq!(reports["spellings"]["exitCode"], 1);
    assert_eq!(reports["suppressions"]["exitCode"], 1);
    assert_eq!(reports["empty"]["exitCode"], 0);
    assert!(reports["empty"]["report"]["rules"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(!reports["configured"]["report"]["rules"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn no_single_enqueue_in_loop_patterns_check_json_snapshot() {
    let root = fixture("pattern-targets");
    let cases = [
        ("scoped", ".no-mistakes.yml", &[13, 14, 17][..]),
        ("mixed", "mixed.yml", &[13, 14, 16, 20][..]),
        ("unscoped", "unscoped.yml", &[13, 14, 17, 19][..]),
    ];
    let fixtures = cases.map(|(label, config, _)| (label, root.clone(), root.join(config)));
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "no_single_enqueue_in_loop_patterns",
        &fixtures,
        &[],
        &[],
    );

    for (label, _, expected) in cases {
        assert_eq!(reports[label]["exitCode"], 1);
        let findings = reports[label]["report"]["rules"].as_array().unwrap();
        let mut lines = findings
            .iter()
            .map(|finding| {
                assert_eq!(finding["file"], "src/publish.ts");
                finding["line"].as_u64().unwrap()
            })
            .collect::<Vec<_>>();
        lines.sort_unstable();
        assert_eq!(lines, expected, "{label}: {findings:#?}");
    }
}
