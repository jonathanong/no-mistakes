use crate::codebase::postgres::embedded::tests::{fixture, read_fixture};
use crate::codebase::postgres::embedded::{
    extract_embedded_sql_from_source, EmbeddedSqlOptions, SqlIdentifierAssertion,
};
#[test]
fn helper_model_resolves_missing_default_for_each_literal_argument() {
    use crate::codebase::postgres::embedded::helpers::{LiteralArg, SiteArg};
    let source = read_fixture("closed-identifiers.ts");
    crate::ast::with_program(
        &fixture("closed-identifiers.ts"),
        &source,
        |program, source| {
            let functions = super::super::resolve::LocalFunctions::collect(program, source, &[]);
            assert!(functions.has_program("column"));
            assert_eq!(
                functions.identifier_clauses(
                    "column",
                    &[SiteArg::Literal(LiteralArg {
                        text: "user".into(),
                        line: 1,
                        origin: 1
                    })]
                ),
                Some(vec!["user_id".into()])
            );
        },
    )
    .unwrap();
}
#[test]
fn closed_identifiers_preserve_record_correlations_and_allowlists() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_sql_identifier_assertions(&[SqlIdentifierAssertion {
            module: "@example/identifiers".into(),
            name: "assertIdentifier".into(),
            allowlist_argument: 1,
        }]);
    let facts = extract_embedded_sql_from_source(
        &fixture("closed-identifiers.ts"),
        &read_fixture("closed-identifiers.ts"),
        &options,
    );
    assert_eq!(facts.calls.len(), 14, "{:?}", facts.calls);
    let texts = |i: usize| {
        facts.calls[i]
            .variants
            .iter()
            .map(|v| v.sql_text.as_str())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        texts(0),
        [
            "DELETE FROM users WHERE user_id = $1",
            "DELETE FROM posts WHERE post_id = $1"
        ]
    );
    assert_eq!(texts(1), ["SELECT * FROM user", "SELECT * FROM post"]);
    assert_eq!(texts(2), ["SELECT * FROM users", "SELECT * FROM posts"]);
    assert_eq!(
        texts(3),
        ["SELECT user_id FROM users", "SELECT post_id FROM users"],
        "{:#?}",
        facts.calls[3]
    );
    assert_eq!(texts(4), ["LOCK TABLE users", "LOCK TABLE posts"]);
    for i in (6..11).chain(std::iter::once(12)) {
        assert_eq!(texts(i), ["LOCK TABLE users", "LOCK TABLE posts"]);
    }
    assert_eq!(
        texts(11),
        ["SELECT user_id FROM users", "SELECT post_id FROM users"]
    );
    assert_eq!(
        texts(13),
        ["SELECT user_id FROM users", "SELECT post_id FROM users"]
    );
    assert!(facts.calls[5].variants.is_empty(), "{:#?}", facts.calls[5]);
}
#[test]
fn unknown_branches_mismatched_imports_and_mutations_remain_opaque() {
    let options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_sql_identifier_assertions(&[SqlIdentifierAssertion {
            module: "@example/identifiers".into(),
            name: "assertIdentifier".into(),
            allowlist_argument: 1,
        }]);
    let facts = extract_embedded_sql_from_source(
        &fixture("closed-identifiers-negative.ts"),
        &read_fixture("closed-identifiers-negative.ts"),
        &options,
    );
    assert_eq!(facts.calls.len(), 14, "{:?}", facts.calls);
    assert!(facts.calls[10..].iter().all(|call| call.is_unanalyzable()));
    for call in facts.calls {
        assert!(
            call.variants.is_empty()
                || call
                    .variants
                    .iter()
                    .all(|variant| variant.sql_text.contains("sql_placeholder")),
            "{call:#?}"
        );
    }
}
#[test]
fn explicitly_raw_tags_enumerate_while_parameterized_tags_keep_bind_values() {
    use crate::codebase::postgres::embedded::TrustedSqlTag;
    let options =
        EmbeddedSqlOptions::configured("@example/db", &[]).with_raw_sql_tags(&[TrustedSqlTag {
            module: "@example/raw-sql".into(),
            name: "sql".into(),
        }]);
    let facts = extract_embedded_sql_from_source(
        &fixture("closed-identifiers-raw-tags.ts"),
        &read_fixture("closed-identifiers-raw-tags.ts"),
        &options,
    );
    assert_eq!(facts.calls.len(), 3);
    assert_eq!(
        facts.calls[0]
            .variants
            .iter()
            .map(|value| value.sql_text.as_str())
            .collect::<Vec<_>>(),
        [
            "DELETE FROM users WHERE user_id = $1",
            "DELETE FROM posts WHERE post_id = $1"
        ]
    );
    assert_eq!(
        facts.calls[1].sql_text.as_deref(),
        Some("SELECT * FROM users WHERE name = sql_placeholder_1")
    );
    assert!(facts.calls[2].variants.is_empty());
    assert!(facts.calls[2].is_unanalyzable());
}
#[test]
fn poisoned_record_alternatives_keep_known_mutating_branches_and_unknown_residual() {
    use crate::codebase::postgres::embedded::TrustedSqlTag;
    let options =
        EmbeddedSqlOptions::configured("@example/db", &[]).with_raw_sql_tags(&[TrustedSqlTag {
            module: "@example/raw-sql".into(),
            name: "sql".into(),
        }]);
    let facts = extract_embedded_sql_from_source(
        &fixture("closed-identifiers-poisoned-branches.ts"),
        &read_fixture("closed-identifiers-poisoned-branches.ts"),
        &options,
    );
    assert_eq!(facts.calls.len(), 1);
    let call = &facts.calls[0];
    assert!(call.is_unanalyzable(), "{call:#?}");
    assert!(
        call.variants
            .iter()
            .any(|variant| variant.sql_text == "DROP TABLE users" && !variant.variants_truncated),
        "{call:#?}"
    );
    assert!(
        call.variants
            .iter()
            .any(|variant| variant.variants_truncated),
        "{call:#?}"
    );
}
#[test]
fn poisoning_is_scoped_to_lexical_record_and_allowlist_declarations() {
    use crate::codebase::postgres::embedded::TrustedSqlTag;
    let options = EmbeddedSqlOptions::configured("@example/db", &[])
        .with_raw_sql_tags(&[TrustedSqlTag {
            module: "@example/raw-sql".into(),
            name: "sql".into(),
        }])
        .with_sql_identifier_assertions(&[SqlIdentifierAssertion {
            module: "@example/identifiers".into(),
            name: "assertIdentifier".into(),
            allowlist_argument: 1,
        }]);
    let facts = extract_embedded_sql_from_source(
        &fixture("closed-identifiers-scope.ts"),
        &read_fixture("closed-identifiers-scope.ts"),
        &options,
    );
    assert_eq!(facts.calls.len(), 5);
    for i in [0, 3] {
        assert!(facts.calls[i].is_unanalyzable(), "{:#?}", facts.calls[i]);
    }
    for i in [1, 2, 4] {
        assert!(!facts.calls[i].is_unanalyzable(), "{:#?}", facts.calls[i]);
        assert_eq!(facts.calls[i].variants.len(), 2, "{:#?}", facts.calls[i]);
    }
}
