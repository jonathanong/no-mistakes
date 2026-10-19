use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-require-query-annotation/fixture")
        .join(name)
}

#[test]
fn shared_project_does_not_double_annotation_profiles() {
    let one = fixture("project-scope-one");
    let two = fixture("project-scope-two");
    let (one_output, one_diagnostics) =
        crate::benchmark_support::check_json_observed(&one, true).unwrap();
    let (two_output, two_diagnostics) =
        crate::benchmark_support::check_json_observed(&two, true).unwrap();
    let one_report: serde_json::Value = serde_json::from_str(&one_output).unwrap();
    let two_report: serde_json::Value = serde_json::from_str(&two_output).unwrap();
    assert!(
        one_report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["rule"] == "postgres-require-query-annotation"),
        "{one_output}"
    );
    assert!(
        two_report["rules"]
            .as_array()
            .unwrap()
            .iter()
            .any(|finding| finding["rule"] == "postgres-require-query-annotation"),
        "{two_output}"
    );
    assert_eq!(
        one_diagnostics.work["postgres.annotation.roots"],
        two_diagnostics.work["postgres.annotation.roots"]
    );
    assert_eq!(
        one_diagnostics.work["postgres.annotation.modules"],
        two_diagnostics.work["postgres.annotation.modules"]
    );
    assert!(one_diagnostics.work["postgres.annotation.roots"] > 0);
    assert!(one_diagnostics.work["postgres.annotation.modules"] > 0);
}
