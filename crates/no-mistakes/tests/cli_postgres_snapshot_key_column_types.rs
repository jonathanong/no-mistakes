use super::check_json_snapshot::{assert_saved_fixture, rules_path};

#[test]
fn postgres_key_column_types_check_json_snapshot() {
    let rule = "postgres-key-column-types";
    // Output cases formerly asserted by the rule unit tests.
    let cases = [
        // must not contain currencies_code_key, partitioned_orders_2025, order_flags, or audit_log
        ("every_bad_key", "mixed", "broad.yml"),
        // partitioned_leaf_external_fkey and text column external_id must remain
        ("partition_leaf_fk", "fail-local-partition-fk", "uuid.yml"),
        // messages must not contain "primary key uses"
        ("primary_keys_off", "mixed", "primary-off.yml"),
        // orders_natural_pkey must remain; messages must not contain foreign key uses
        ("foreign_keys_off_uuid", "mixed", "foreign-off-uuid.yml"),
        // character varying column locale must remain; messages must not contain foreign key uses
        (
            "foreign_keys_off_varchar",
            "mixed",
            "foreign-off-varchar.yml",
        ),
        // order_flags_pkey and order_id_domain must remain; the order_id_domain finding must not contain "string"
        ("enum_off_and_domains", "mixed", "uuid.yml"),
        (
            "cross_schema_enum_allowed",
            "cross-schema-enum",
            "enum-on.yml",
        ),
        // external_enum_keys_pkey must remain
        (
            "cross_schema_enum_rejected",
            "cross-schema-enum",
            "uuid.yml",
        ),
        // domain_priority_keys_pkey must remain
        (
            "external_enum_local_domain",
            "external-enum-shadow",
            "enum-on.yml",
        ),
        (
            "selected_schema_enum",
            "selected-schema-enum",
            "enum-on.yml",
        ),
        // must not contain locale_keys.locale_keys_pkey
        ("typmod_allowed_exactly", "mixed", "typmod-allowed.yml"),
        // custom(12, custom(foo), and numeric(10,2) must remain
        ("display_type_stripping", "display-types", "uuid.yml"),
        // every message must end with review this key
        ("custom_rule_message", "mixed", "review-message.yml"),
        ("include_skips_catalog", "mixed", "include-migrations.yml"),
    ]
    .map(|(label, fixture, config)| {
        (
            label,
            rules_path(rule, "fixture", fixture),
            rules_path(rule, "configs", config),
        )
    });
    assert_saved_fixture("postgres_key_column_types", &cases);
}
