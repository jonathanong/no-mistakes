use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "test-fixture-size-cap";

#[test]
fn test_fixture_size_cap_json_matches_snapshot() {
    let policy = rules_path(RULE, "configs", "policy.yml");
    assert_saved_fixture(
        "test_fixture_size_cap",
        &[
            // Oversized Array.from, new Array, and it.each literals in tests
            // and helpers. Boundary, exempt, record, and production files stay quiet.
            (
                "policy",
                rules_path(RULE, "fixture", "policy"),
                policy.clone(),
            ),
            // 0 findings: every literal is at the threshold, exempt, or a record table.
            ("pass", rules_path(RULE, "fixture", "pass"), policy.clone()),
            // The 9000 length remains. Next-line and same-line directives hide the others.
            (
                "suppressed",
                rules_path(RULE, "fixture", "suppressed"),
                policy.clone(),
            ),
            // 0 findings: a file directive hides every literal in the file.
            (
                "file_disabled",
                rules_path(RULE, "fixture", "file-disabled"),
                policy.clone(),
            ),
            // Bare Array(N) matches new Array(N). Mixed lists report numeric literals.
            (
                "recall",
                rules_path(RULE, "fixture", "recall"),
                policy.clone(),
            ),
            // Later getters, setters, spreads, and dynamic keys hide an earlier length.
            // A known non-length key and a length after a spread stay reported.
            (
                "dynamic_length",
                rules_path(RULE, "fixture", "dynamic-length"),
                policy,
            ),
            // 0 findings: omitting threshold reports nothing.
            (
                "no_threshold",
                rules_path(RULE, "fixture", "policy"),
                rules_path(RULE, "configs", "no-threshold.yml"),
            ),
            // 1 finding: the configured message prefixes the cardinality text.
            (
                "message",
                rules_path(RULE, "fixture", "message"),
                rules_path(RULE, "configs", "message.yml"),
            ),
        ],
    );
}
