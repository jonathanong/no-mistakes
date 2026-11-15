use crate::codebase::postgres::embedded::tests::{fixture, read_fixture};
use crate::codebase::postgres::embedded::{
    extract_embedded_sql_from_source, EmbeddedSqlCall, EmbeddedSqlOptions,
};

pub(super) fn calls(name: &str) -> Vec<EmbeddedSqlCall> {
    extract_embedded_sql_from_source(
        &fixture(name),
        &read_fixture(name),
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    )
    .calls
}

pub(super) fn sqls(calls: &[EmbeddedSqlCall], marker: &str) -> Vec<String> {
    let call = calls
        .iter()
        .find(|call| {
            call.sql_text
                .as_deref()
                .is_some_and(|sql| sql.contains(marker))
                || call
                    .variants
                    .iter()
                    .any(|variant| variant.sql_text.contains(marker))
        })
        .unwrap_or_else(|| panic!("missing {marker}: {calls:#?}"));
    if call.variants.is_empty() {
        call.sql_text.iter().cloned().collect()
    } else {
        call.variants
            .iter()
            .map(|variant| variant.sql_text.clone())
            .collect()
    }
}

#[test]
fn hasown_frozen_records_enumerate_strings_and_keep_nested_fields_together() {
    let calls = calls("hasown-identifiers.ts");
    assert_eq!(
        sqls(&calls, "/* both */"),
        [
            "/* both */ DELETE FROM users WHERE id = $1",
            "/* both */ DELETE FROM posts WHERE id = $1",
        ]
    );
    assert_eq!(
        sqls(&calls, "/* corr */"),
        [
            "/* corr */ DELETE FROM users WHERE user_id = $1",
            "/* corr */ DELETE FROM posts WHERE post_id = $1",
        ]
    );
    assert_eq!(
        sqls(&calls, "/* one */"),
        ["/* one */ DELETE FROM users WHERE id = $1"]
    );
    assert_eq!(
        sqls(&calls, "/* append */"),
        [
            "/* append */ DELETE FROM users WHERE id = $1",
            "/* append */ DELETE FROM posts WHERE id = $1",
        ]
    );
    let missing = calls
        .iter()
        .find(|call| {
            call.sql_text
                .as_deref()
                .is_some_and(|sql| sql.contains("/* miss */"))
                || call
                    .variants
                    .iter()
                    .any(|variant| variant.sql_text.contains("/* miss */"))
        })
        .unwrap_or_else(|| panic!("missing miss call: {calls:#?}"));
    assert!(missing.is_unanalyzable(), "{missing:#?}");
    assert_eq!(
        sqls(&calls, "/* one-config */"),
        ["/* one-config */ DELETE FROM users WHERE user_id = $1"]
    );
    assert_eq!(
        sqls(&calls, "/* union-config */"),
        [
            "/* union-config */ DELETE FROM users WHERE user_id = $1",
            "/* union-config */ DELETE FROM posts WHERE post_id = $1",
        ]
    );
    for marker in ["/* partial-config */", "/* object-key */", "/* optional */"] {
        let call = calls
            .iter()
            .find(|call| {
                call.sql_text
                    .as_deref()
                    .is_some_and(|sql| sql.contains(marker))
                    || call
                        .variants
                        .iter()
                        .any(|variant| variant.sql_text.contains(marker))
            })
            .unwrap_or_else(|| panic!("missing {marker}: {calls:#?}"));
        assert!(call.is_unanalyzable(), "{marker}: {call:#?}");
    }
}

#[test]
fn hasown_rejects_unfrozen_mutated_escaped_and_oversized_records() {
    let calls = calls("hasown-opaque.ts");
    for marker in [
        "/* open */",
        "/* quiet */",
        "/* extra */",
        "/* mutable */",
        "/* indexed */",
        "/* escaped */",
        "/* exported */",
        "/* spread */",
        "/* mixed */",
        "/* shadowed */",
        "/* many */",
    ] {
        let call = calls
            .iter()
            .find(|call| {
                call.sql_text
                    .as_deref()
                    .is_some_and(|sql| sql.contains(marker))
                    || call
                        .variants
                        .iter()
                        .any(|variant| variant.sql_text.contains(marker))
            })
            .unwrap_or_else(|| panic!("missing {marker}: {calls:#?}"));
        assert!(call.is_unanalyzable(), "{marker}: {call:#?}");
    }
    assert_eq!(
        sqls(&calls, "/* clean */"),
        [
            "/* clean */ DELETE FROM users WHERE id = $1",
            "/* clean */ DELETE FROM posts WHERE id = $1",
        ]
    );
}

#[test]
fn hasown_drops_every_record_when_object_is_shadowed() {
    let calls = calls("hasown-shadow.ts");
    let call = calls
        .iter()
        .find(|call| {
            call.sql_text
                .as_deref()
                .is_some_and(|sql| sql.contains("/* shadow */"))
                || call
                    .variants
                    .iter()
                    .any(|variant| variant.sql_text.contains("/* shadow */"))
        })
        .unwrap_or_else(|| panic!("missing shadow call: {calls:#?}"));
    assert!(call.is_unanalyzable(), "{call:#?}");
}
