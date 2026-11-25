use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn csp_allowed_origins_json_matches_snapshot() {
    let report = rules_path("csp-allowed-origins", "fixture", "report");
    let headers_text = rules_path("csp-allowed-origins", "fixture", "headers-text");
    assert_saved_fixture(
        "csp_allowed_origins",
        &[
            ("report", report.clone(), report.join(".no-mistakes.yml")),
            (
                "default-off",
                report.clone(),
                report.join("default-off.yml"),
            ),
            (
                "headers-text",
                headers_text.clone(),
                headers_text.join(".no-mistakes.yml"),
            ),
        ],
    );
}
