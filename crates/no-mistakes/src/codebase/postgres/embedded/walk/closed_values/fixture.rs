use crate::codebase::postgres::embedded::{
    extract_embedded_sql_from_source, EmbeddedSqlCall, EmbeddedSqlOptions, TrustedSqlTag,
};
use std::path::PathBuf;

fn load(file: &str) -> (PathBuf, String, Vec<EmbeddedSqlCall>) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../test-cases/rules/postgres-bounded-statements/closed-identifier-correlation/fixture")
        .join(file);
    let source = std::fs::read_to_string(&path).expect("fixture");
    let options =
        EmbeddedSqlOptions::configured("@example/db", &[]).with_raw_sql_tags(&[TrustedSqlTag {
            module: "@example/raw-sql".into(),
            name: "sql".into(),
        }]);
    let calls = extract_embedded_sql_from_source(&path, &source, &options).calls;
    (path, source, calls)
}

pub(super) fn call(file: &str, index: usize) -> EmbeddedSqlCall {
    load(file).2.into_iter().nth(index).expect("call")
}

pub(super) fn call_containing(file: &str, needle: &str) -> EmbeddedSqlCall {
    let (_, source, calls) = load(file);
    let line = source
        .lines()
        .position(|line| line.contains(needle))
        .expect("needle") as u32
        + 1;
    calls
        .into_iter()
        .find(|call| call.line == line)
        .expect("call")
}

pub(super) fn texts(file: &str, index: usize) -> Vec<String> {
    sql_text(call(file, index))
}

pub(super) fn texts_containing(file: &str, needle: &str) -> Vec<String> {
    sql_text(call_containing(file, needle))
}

fn sql_text(call: EmbeddedSqlCall) -> Vec<String> {
    call.variants
        .iter()
        .map(|variant| variant.sql_text.clone())
        .collect()
}
