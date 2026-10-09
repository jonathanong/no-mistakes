use super::super::{compile::compile, scan::scan, Options};
use super::support::{findings, PATH};
use crate::codebase::postgres::SchemaCatalog;
use std::path::PathBuf;

#[test]
fn reports_every_bad_key_once_in_constraint_column_order() {
    let messages = findings(&format!(
        "{PATH}allowedTypes: [uuid, bigint, integer, smallint]\nallowEnumTypes: true\n"
    ))
    .into_iter()
    .map(|finding| finding.message)
    .collect::<Vec<_>>();
    assert!(messages.iter().any(|message| message == "schema.json: constraint:orders.orders_natural_pkey: primary key uses text column code; use one of uuid, bigint, integer, smallint, or an enum, and keep any natural key as a separate unique constraint"), "{messages:#?}");
    assert!(messages.iter().any(|message| message == "schema.json: constraint:order_notes.order_notes_pkey: primary key uses text column author_email; use one of uuid, bigint, integer, smallint, or an enum, and keep any natural key as a separate unique constraint"), "{messages:#?}");
    assert!(messages.iter().any(|message| message == "schema.json: constraint:orders.orders_currency_code_fkey: foreign key uses text column currency_code (references currencies); review the referenced key's type and use a compatible configured type"), "{messages:#?}");
    assert!(messages.iter().any(|message| message.contains("constraint:missing_key_column.missing_key_column_pkey: primary key uses unknown column missing_id")), "missing catalog metadata must fail closed: {messages:#?}");
    assert!(
        !messages
            .iter()
            .any(|message| message.contains("partitioned_orders_2025")),
        "{messages:#?}"
    );
    assert!(
        !messages
            .iter()
            .any(|message| message.contains("order_flags")),
        "{messages:#?}"
    );
    assert!(
        !messages.iter().any(|message| message.contains("audit_log")),
        "{messages:#?}"
    );
}

#[test]
fn key_flags_keep_primary_and_foreign_checks_scoped() {
    let messages = findings(&format!(
        "{PATH}allowedTypes: [uuid]\ncheckPrimaryKeys: false\n"
    ))
    .into_iter()
    .map(|finding| finding.message)
    .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .all(|message| message.contains("foreign key uses")),
        "{messages:#?}"
    );
    let messages = findings(&format!(
        "{PATH}allowedTypes: [UUID]\ncheckForeignKeys: false\n"
    ))
    .into_iter()
    .map(|finding| finding.message)
    .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("orders_natural_pkey")),
        "{messages:#?}"
    );
    let messages = findings(&format!(
        "{PATH}allowedTypes: [uuid, varchar]\ncheckForeignKeys: false\n"
    ))
    .into_iter()
    .map(|finding| finding.message)
    .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("character varying column locale")),
        "{messages:#?}"
    );
}

#[test]
fn enum_allowance_is_explicit_and_domains_do_not_inherit_allowed_base_types() {
    let messages = findings(&format!("{PATH}allowedTypes: [uuid]\n"))
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("order_flags.order_flags_pkey")),
        "enum key fails without opt-in: {messages:#?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("primary key uses order_id_domain column id")),
        "domain over an allowed base type remains disallowed: {messages:#?}"
    );
    assert!(
        messages.iter().any(|message| {
            message.contains("order_id_domain column id")
                && message.contains("keep any natural key as a separate unique constraint")
                && !message.contains("string")
        }),
        "advice should not assume the rejected key type is text: {messages:#?}"
    );
}

#[test]
fn allowed_types_match_catalog_values_before_message_display_normalization() {
    let allowed_messages = findings(&format!(
        "{PATH}allowedTypes: [UUID, 'CHARACTER VARYING(32)']\n"
    ))
    .into_iter()
    .map(|finding| finding.message)
    .collect::<Vec<_>>();
    assert!(
        !allowed_messages
            .iter()
            .any(|message| message.contains("locale_keys.locale_keys_pkey")),
        "case-insensitive exact catalog type should allow the typmod value: {allowed_messages:#?}"
    );
    let rejected_messages = findings(&format!("{PATH}allowedTypes: [uuid]\n"))
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>();
    assert!(
        rejected_messages
            .iter()
            .any(|message| message.contains("character varying column locale")),
        "the same type is displayed without numeric typmod when disallowed: {rejected_messages:#?}"
    );
}

#[test]
fn refuses_to_guess_primary_key_constraint_identity_without_primary_index_metadata() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/key-column-types/catalogs/missing-primary-index.json");
    let source = std::fs::read_to_string(path).unwrap();
    let catalog = SchemaCatalog::from_json(&source).unwrap();
    let options: Options =
        serde_yaml::from_str("schemaCatalogPath: schema.json\nallowedTypes: [uuid]\n").unwrap();
    let compiled = compile(&options, None).unwrap();
    let error = scan(&catalog, &compiled, "schema.json").unwrap_err();
    assert!(
        error
            .to_string()
            .contains("has a primary key but no primary index name"),
        "{error:#}"
    );
    assert!(
        error.to_string().contains("regenerate the schema catalog"),
        "{error:#}"
    );
}

#[test]
fn preserves_type_strings_with_unrecognized_modifiers_in_messages() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/key-column-types/catalogs/display-types.json");
    let source = std::fs::read_to_string(path).unwrap();
    let catalog = SchemaCatalog::from_json(&source).unwrap();
    let options: Options =
        serde_yaml::from_str("schemaCatalogPath: schema.json\nallowedTypes: [uuid]\n").unwrap();
    let compiled = compile(&options, None).unwrap();
    let messages = scan(&catalog, &compiled, "schema.json")
        .unwrap()
        .into_iter()
        .map(|finding| finding.message)
        .collect::<Vec<_>>();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("custom(12 column id")),
        "{messages:#?}"
    );
    assert!(
        messages
            .iter()
            .any(|message| message.contains("custom(foo) column id")),
        "{messages:#?}"
    );
}

#[test]
fn applies_rule_message_to_each_catalog_finding() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres/key-column-types/catalogs/mixed.json");
    let source = std::fs::read_to_string(path).unwrap();
    let catalog = SchemaCatalog::from_json(&source).unwrap();
    let options: Options =
        serde_yaml::from_str("schemaCatalogPath: schema.json\nallowedTypes: [uuid]\n").unwrap();
    let compiled = compile(&options, Some("review this key".into())).unwrap();
    let messages = scan(&catalog, &compiled, "schema.json").unwrap();
    assert!(messages
        .iter()
        .all(|finding| finding.message.ends_with("review this key")));
}
