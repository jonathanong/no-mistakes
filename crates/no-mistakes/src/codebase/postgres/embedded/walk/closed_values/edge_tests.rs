use crate::codebase::postgres::embedded::{
    extract_embedded_sql_from_source, EmbeddedSqlOptions, TrustedSqlTag,
};
use std::path::PathBuf;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-bounded-statements/closed-identifier-correlation/fixture/edges.ts",
    )
}

fn texts(index: usize) -> Vec<String> {
    let path = fixture_path();
    let source = std::fs::read_to_string(&path).expect("fixture");
    let options =
        EmbeddedSqlOptions::configured("@example/db", &[]).with_raw_sql_tags(&[TrustedSqlTag {
            module: "@example/raw-sql".into(),
            name: "sql".into(),
        }]);
    let facts = extract_embedded_sql_from_source(&path, &source, &options);
    facts.calls[index]
        .variants
        .iter()
        .map(|variant| variant.sql_text.clone())
        .collect()
}

#[test]
fn same_argument_keeps_only_the_diagonal() {
    assert_eq!(
        texts(0),
        [
            "DELETE FROM users WHERE id = $1",
            "DELETE FROM posts WHERE id = $1"
        ]
    );
}

#[test]
fn two_closed_arguments_publish_their_product() {
    assert_eq!(
        texts(1),
        [
            "DELETE FROM a_w WHERE id = $1",
            "DELETE FROM a_x WHERE id = $1",
            "DELETE FROM b_w WHERE id = $1",
            "DELETE FROM b_x WHERE id = $1"
        ]
    );
}

#[test]
fn product_over_the_variant_budget_stays_opaque() {
    let values = texts(2);
    assert!(values.is_empty(), "{values:?}");
}

#[test]
fn const_object_argument_does_not_expand_its_fields() {
    assert_eq!(texts(3), ["DELETE FROM literal WHERE id = $1"]);
}
