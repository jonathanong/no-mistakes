use super::check_json_snapshot::assert_saved_fixture_report;
use std::path::PathBuf;

#[test]
fn postgres_annotation_call_values_check_json_snapshot() {
    let fixtures = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/performance");
    let scalar_refresh = "annotation-scalar-call-values/scalar-refresh";
    let scalar_opaque = "annotation-scalar-call-values/scalar-opaque";
    let cases = [
        (
            "shared_callback_payload_low",
            "annotation-fact-scale",
            "low.yml",
        ),
        (
            "shared_callback_payload_max",
            "annotation-fact-scale",
            "max.yml",
        ),
        ("scalar_refresh_low", scalar_refresh, "low.yml"),
        ("scalar_refresh_max", scalar_refresh, "max.yml"),
        ("scalar_opaque_low", scalar_opaque, "low.yml"),
        ("scalar_opaque_max", scalar_opaque, "max.yml"),
    ]
    .map(|(label, fixture, config)| {
        let root = fixtures.join(fixture);
        (label, root.clone(), root.join(config))
    });
    let reports = assert_saved_fixture_report("postgres_annotation_call_values", &cases, &[], &[]);
    for [(low, ..), (max, ..)] in cases.as_chunks::<2>().0 {
        assert_eq!(reports[*low], reports[*max], "{low} and {max}");
    }
}
