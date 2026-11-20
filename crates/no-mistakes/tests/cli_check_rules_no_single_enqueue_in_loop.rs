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
    let reports = crate::check_json_snapshot::assert_saved_fixture_report(
        "no_single_enqueue_in_loop",
        &[
            (
                "configured",
                configured.clone(),
                configured.join(".no-mistakes.yml"),
            ),
            ("empty", empty.clone(), empty.join(".no-mistakes.yml")),
        ],
        &[],
        &[],
    );
    assert_eq!(reports["configured"]["exitCode"], 1);
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
