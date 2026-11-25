use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn csp_allowed_origins_json_matches_snapshot() {
    let report = rules_path("csp-allowed-origins", "fixture", "report");
    let headers_text = rules_path("csp-allowed-origins", "fixture", "headers-text");
    let browser_semantics = rules_path("csp-allowed-origins", "fixture", "browser-semantics");
    let unanalyzable = rules_path("csp-allowed-origins", "fixture", "unanalyzable-and-urls");
    let yaml_block = rules_path("csp-allowed-origins", "fixture", "yaml-block");
    let dynamic_policies = rules_path("csp-allowed-origins", "fixture", "dynamic-policies");
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
            (
                "duplicate-directive",
                browser_semantics.clone(),
                browser_semantics.join("duplicate.yml"),
            ),
            (
                "multi-policy-strings",
                browser_semantics.clone(),
                browser_semantics.join("multi-string.yml"),
            ),
            (
                "multi-policy-files",
                browser_semantics.clone(),
                browser_semantics.join("multi-file.yml"),
            ),
            (
                "unanalyzable-report",
                unanalyzable.clone(),
                unanalyzable.join("report.yml"),
            ),
            (
                "unanalyzable-advise",
                unanalyzable.clone(),
                unanalyzable.join("advise.yml"),
            ),
            (
                "unanalyzable-ignore",
                unanalyzable.clone(),
                unanalyzable.join("ignore.yml"),
            ),
            (
                "yaml-block",
                yaml_block.clone(),
                yaml_block.join(".no-mistakes.yml"),
            ),
            (
                "dynamic-nonce-join",
                dynamic_policies.clone(),
                dynamic_policies.join("nonce-join.yml"),
            ),
            (
                "dynamic-join-report",
                dynamic_policies.clone(),
                dynamic_policies.join("dynamic-join-report.yml"),
            ),
            (
                "dynamic-join-advise",
                dynamic_policies.clone(),
                dynamic_policies.join("dynamic-join-advise.yml"),
            ),
            (
                "dynamic-ternary",
                dynamic_policies.clone(),
                dynamic_policies.join("ternary.yml"),
            ),
        ],
    );
}
