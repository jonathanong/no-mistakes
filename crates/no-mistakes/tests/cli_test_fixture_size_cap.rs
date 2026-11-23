use super::check_json_snapshot::{assert_saved_fixture, rules_path};

const RULE: &str = "test-fixture-size-cap";

#[test]
fn test_fixture_size_cap_json_matches_snapshot() {
    let policy = rules_path(RULE, "configs", "policy.yml");
    let each = rules_path(RULE, "configs", "it-each.yml");
    assert_saved_fixture(
        "test_fixture_size_cap",
        &[
            // Oversized Array.from and new Array literals in tests and helpers.
            // Boundary, exempt, record, production, and unconfigured each tables stay quiet.
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
            // Bare Array(N) matches new Array(N). Configured it.each reports numeric literals.
            (
                "recall",
                rules_path(RULE, "fixture", "recall"),
                each.clone(),
            ),
            // Boundary-value tables are inputs under the default empty eachCalls list.
            (
                "boundary",
                rules_path(RULE, "fixture", "boundary"),
                policy.clone(),
            ),
            // The same integers report once eachCalls lists it.each.
            (
                "boundary_each",
                rules_path(RULE, "fixture", "boundary"),
                each,
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

#[test]
fn test_fixture_size_cap_each_table_mode_json_matches_snapshot() {
    let fixture = rules_path(RULE, "fixture", "each-table-mode");
    assert_saved_fixture(
        "test_fixture_size_cap_each_table_mode",
        &[
            (
                "all_elements",
                fixture.clone(),
                rules_path(RULE, "configs", "all-elements.yml"),
            ),
            (
                "default",
                fixture,
                rules_path(RULE, "configs", "it-each.yml"),
            ),
        ],
    );
}
