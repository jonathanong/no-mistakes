use crate::codebase::postgres::embedded::{
    extract_embedded_sql_from_source, EmbeddedSqlCall, EmbeddedSqlOptions, TrustedSqlTag,
};
use std::path::PathBuf;

pub(super) fn call(file: &str, index: usize) -> EmbeddedSqlCall {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/closed-identifier-correlation/fixture")
        .join(file);
    let source = std::fs::read_to_string(&path).expect("fixture");
    let options =
        EmbeddedSqlOptions::configured("@example/db", &[]).with_raw_sql_tags(&[TrustedSqlTag {
            module: "@example/raw-sql".into(),
            name: "sql".into(),
        }]);
    extract_embedded_sql_from_source(&path, &source, &options).calls[index].clone()
}

pub(super) fn texts(file: &str, index: usize) -> Vec<String> {
    call(file, index)
        .variants
        .iter()
        .map(|variant| variant.sql_text.clone())
        .collect()
}
