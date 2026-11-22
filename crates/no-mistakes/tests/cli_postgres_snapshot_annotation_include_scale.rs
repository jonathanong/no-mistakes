use super::check_json_snapshot::assert_saved_fixture;

#[test]
fn annotation_include_scale_check_json_snapshot() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/performance/annotation-include-scale");
    let mut cases = ["small-max", "small-low"]
        .map(|name| {
            (
                name,
                root.clone(),
                root.join("configs").join(format!("{name}.yml")),
            )
        })
        .to_vec();
    let intrinsics = root.parent().unwrap().join("annotation-include-intrinsics");
    cases.push((
        "global-intrinsics",
        intrinsics.clone(),
        intrinsics.join(".no-mistakes.yml"),
    ));
    let failed_source = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/scoped-source-prerequisite",
    );
    for (label, config) in [
        ("generic-include-source-prerequisite", "no-mistakes.yaml"),
        ("option-include-source-prerequisite", "options-control.yaml"),
    ] {
        cases.push((label, failed_source.clone(), failed_source.join(config)));
    }
    assert_saved_fixture("postgres_annotation_include_scale", &cases);
}
