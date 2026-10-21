use super::*;
use crate::codebase::postgres::embedded::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use std::path::PathBuf;

#[test]
fn unknown_callees_skip_argument_classification() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded/identifier-clause-lazy-args.ts");
    let source = std::fs::read_to_string(&path).expect("saved fixture");
    reset_arg_classifications();

    let calls = extract_embedded_sql_from_source(
        &path,
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    )
    .calls;
    assert_eq!(calls.len(), 2, "{calls:#?}");
    assert_eq!(
        arg_classifications(),
        1,
        "only the known helper classifies args"
    );
    assert_eq!(calls[0].variants.len(), 1, "known helpers still recover");
    assert!(calls[0].variants[0].sql_text.contains("FROM orders"));
    assert!(calls[1].is_unanalyzable(), "{:#?}", calls[1]);
}
