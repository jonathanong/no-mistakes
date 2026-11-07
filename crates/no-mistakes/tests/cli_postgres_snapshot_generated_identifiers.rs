use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_generated_identifiers_check_json_snapshot() {
    let rule = "postgres-no-unbounded-sql-identifier";
    let config = rules_path(rule, "configs", "registry.yml");
    assert_saved_fixture(
        "postgres_generated_identifiers",
        &[
            (
                "pass_hex",
                rules_path(rule, "fixture", "pass-generated-hex-name"),
                config.clone(),
            ),
            (
                "unproven_hex_shapes",
                rules_path(rule, "fixture", "fail-generated-hex-shapes"),
                config.clone(),
            ),
            (
                "unproven_hex_trust",
                rules_path(rule, "fixture", "fail-generated-hex-trust"),
                config,
            ),
        ],
    );
}
