use super::check_json_snapshot::{assert_saved_fixture, rules_path};
use std::path::PathBuf;

fn repo_rel(rel: &str) -> PathBuf {
    no_mistakes::codebase::ts_resolver::normalize_path(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(rel),
    )
}

#[test]
fn postgres_array_columns_check_json_snapshot() {
    let rule = "postgres-array-columns";
    let catalog = "catalog.yml";
    let cases = [
        ("text_array", "orders-tag-names", catalog),
        // Messages must not contain stale.
        (
            "uuid_allow_does_not_excuse",
            "orders-tags-and-coupons",
            "allow-coupon.yml",
        ),
        (
            "oauth_redirect_allow",
            "oauth-redirect-uris",
            "allow-oauth.yml",
        ),
        ("enum_elements", "report-types", "allow-enum.yml"),
        (
            "schema_qualified_enum_array",
            "audit-report-types",
            "allow-enum.yml",
        ),
        ("weekday_enum_allowed", "weekday-enum", "allow-enum.yml"),
        ("weekday_enum", "weekday-enum", catalog),
        ("smallint_element", "sensor-samples", "allow-smallint.yml"),
        (
            "external_enum_domain",
            "external-enum-domain-shadow",
            "allow-enum.yml",
        ),
        (
            "integer_multidimensional_allowed",
            "metric-buckets",
            "allow-integer.yml",
        ),
        ("multidimensional_integer", "metric-buckets", catalog),
        ("spaced_element_type", "label-names", catalog),
        (
            "character_varying_allowed",
            "label-names",
            "allow-character-varying.yml",
        ),
        ("mixed_case_uuid", "coupon-ids-mixed-case", catalog),
        ("nested_uuid", "coupon-ids-nested", catalog),
        ("non_array", "orders-note", catalog),
        ("uuid_never_allow", "coupon-ids", catalog),
        ("empty_never_allow", "coupon-ids", "empty-never.yml"),
        ("stale_allow", "accounts-display-name", "stale-allow.yml"),
        ("custom_message", "fail", "custom-message.yml"),
        ("include_misses_catalog", "fail", "include-missing.yml"),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(rule, "fixture", fixture),
            rules_path(rule, "configs", config),
        )
    });
    let mut cases = cases.to_vec();
    cases.insert(
        0,
        (
            "prepared_catalog",
            repo_rel("fixtures/postgres/array-columns/pass"),
            rules_path(rule, "configs", catalog),
        ),
    );
    assert_saved_fixture("postgres_array_columns", &cases);
}
