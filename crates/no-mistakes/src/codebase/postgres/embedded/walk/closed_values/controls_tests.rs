use crate::codebase::postgres::embedded::tests::{fixture, read_fixture};
use crate::codebase::postgres::embedded::{
    extract_embedded_sql_from_source, EmbeddedSqlOptions, SqlIdentifierAssertion, TrustedSqlTag,
};

#[test]
fn unsupported_identifier_and_raw_tag_forms_never_publish_complete_variants() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_sql_identifier_assertions(&[
            SqlIdentifierAssertion {
                module: "@example/identifiers".into(),
                name: "assertIdentifier".into(),
                allowlist_argument: 1,
            },
            SqlIdentifierAssertion {
                module: "@example/identifiers".into(),
                name: "assertAtTwo".into(),
                allowlist_argument: 2,
            },
        ])
        .with_raw_sql_tags(&[TrustedSqlTag {
            module: "@example/raw-sql".into(),
            name: "sql".into(),
        }]);
    let facts = extract_embedded_sql_from_source(
        &fixture("closed-identifiers-controls.ts"),
        &read_fixture("closed-identifiers-controls.ts"),
        &options,
    );
    assert_eq!(facts.calls.len(), 29, "{:#?}", facts.calls);
    for call in &facts.calls {
        assert!(call.is_unanalyzable(), "line {}: {call:#?}", call.line);
        assert!(
            call.variants
                .iter()
                .all(|variant| variant.variants_truncated),
            "line {} published a complete alternative: {call:#?}",
            call.line
        );
    }
}
