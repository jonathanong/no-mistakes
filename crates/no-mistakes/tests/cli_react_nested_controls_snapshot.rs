use super::check_json_snapshot::{assert_saved_fixture_report, rules_path};

const RULE: &str = "react-no-nested-buttons-and-links";

#[test]
fn check_json_snapshot_native_button_and_link_nesting() {
    let reports = assert_saved_fixture_report(
        "react_no_nested_buttons_and_links",
        &[
            (
                "configured",
                rules_path(RULE, "fixture", "."),
                rules_path(RULE, "configs", "check.yml"),
            ),
            (
                "unanalyzable_advise",
                rules_path(RULE, "fixture", "."),
                rules_path(RULE, "configs", "unanalyzable-advise.yml"),
            ),
            (
                "unanalyzable_ignore",
                rules_path(RULE, "fixture", "."),
                rules_path(RULE, "configs", "unanalyzable-ignore.yml"),
            ),
            (
                "parse_error",
                rules_path(RULE, "parse-error", "."),
                rules_path(RULE, "configs", "parse-error.yml"),
            ),
            (
                "default_off",
                rules_path(RULE, "fixture", "."),
                rules_path(RULE, "configs", "default-off.yml"),
            ),
        ],
        &[],
        &[],
    );
    let configured = &reports["configured"];
    assert_eq!(configured["exitCode"], 1);
    let findings = configured["report"]["rules"].as_array().unwrap();
    assert_eq!(findings.len(), 14);
    assert!(findings.iter().all(|finding| finding["rule"] == RULE));
    assert_eq!(
        reports["unanalyzable_advise"]["report"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        reports["unanalyzable_advise"]["report"]["advisories"]
            .as_array()
            .unwrap()
            .len(),
        10
    );
    assert_eq!(
        reports["unanalyzable_ignore"]["report"]["rules"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
    assert_eq!(
        reports["unanalyzable_ignore"]["report"]["advisories"],
        serde_json::json!([])
    );
    assert_eq!(
        reports["parse_error"]["report"]["advisories"],
        serde_json::json!([])
    );
    assert_eq!(reports["default_off"]["exitCode"], 0);
    assert_eq!(
        reports["default_off"]["report"]["rules"],
        serde_json::json!([])
    );
}
