use crate::codebase::postgres::embedded::{
    extract_embedded_sql_from_source, EmbeddedSqlOptions, TrustedSqlTag,
};
use std::path::PathBuf;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-bounded-statements/closed-identifier-correlation/fixture/query.ts",
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
fn record_and_helper_keep_the_argument_choice() {
    assert_eq!(
        texts(0),
        [
            "DELETE FROM users WHERE user_id = $1",
            "DELETE FROM posts WHERE post_id = $1"
        ]
    );
}

#[test]
fn two_helpers_keep_the_same_argument_choice() {
    assert_eq!(
        texts(1),
        [
            "DELETE FROM users WHERE user_id = $1",
            "DELETE FROM posts WHERE post_id = $1"
        ]
    );
}

#[test]
fn distinct_parameters_keep_the_cross_pairs() {
    let mut values = texts(2);
    values.sort();
    assert_eq!(
        values,
        [
            "DELETE FROM posts WHERE post_id = $1",
            "DELETE FROM posts WHERE user_id = $1",
            "DELETE FROM users WHERE post_id = $1",
            "DELETE FROM users WHERE user_id = $1"
        ]
    );
}
