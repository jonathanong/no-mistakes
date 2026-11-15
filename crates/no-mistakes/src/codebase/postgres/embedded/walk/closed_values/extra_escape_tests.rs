use crate::codebase::postgres::embedded::tests::{fixture, read_fixture};
use crate::codebase::postgres::embedded::{
    extract_embedded_sql_from_source, EmbeddedSqlOptions, TrustedSqlTag,
};

#[test]
fn stored_wrappers_yield_and_throw_open_nested_record_values() {
    let options =
        EmbeddedSqlOptions::configured("@example/db", &[]).with_raw_sql_tags(&[TrustedSqlTag {
            module: "@example/raw-sql".into(),
            name: "sql".into(),
        }]);
    let facts = extract_embedded_sql_from_source(
        &fixture("closed-identifiers-extra-escapes.ts"),
        &read_fixture("closed-identifiers-extra-escapes.ts"),
        &options,
    );
    assert_eq!(facts.calls.len(), 5, "{:#?}", facts.calls);
    for call in &facts.calls {
        assert!(call.is_unanalyzable(), "{call:#?}");
        assert!(
            call.variants
                .iter()
                .all(|variant| variant.variants_truncated),
            "{call:#?}"
        );
    }
}

#[test]
fn class_parameter_and_jsx_handoffs_open_objects_but_keep_scalar_reads_closed() {
    let options =
        EmbeddedSqlOptions::configured("@example/db", &[]).with_raw_sql_tags(&[TrustedSqlTag {
            module: "@example/raw-sql".into(),
            name: "sql".into(),
        }]);
    for (name, unknown, total) in [
        ("closed-identifiers-initializers.ts", 4, 6),
        ("closed-identifiers-jsx.tsx", 1, 2),
    ] {
        let facts = extract_embedded_sql_from_source(&fixture(name), &read_fixture(name), &options);
        assert_eq!(facts.calls.len(), total, "{name}: {:#?}", facts.calls);
        for call in &facts.calls[..unknown] {
            assert!(call.is_unanalyzable(), "{name}: {call:#?}");
        }
        for call in &facts.calls[unknown..] {
            assert!(!call.is_unanalyzable(), "{name}: {call:#?}");
            assert_eq!(call.variants.len(), 2);
        }
    }
}
