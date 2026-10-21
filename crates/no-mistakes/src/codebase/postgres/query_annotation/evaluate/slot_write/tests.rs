use super::super::{Evaluator, File, Value};
use crate::codebase::postgres::query_annotation::evaluate::ArgumentObjects;
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{fx_map, FxHashMap};
use std::path::PathBuf;

#[path = "recreated_tests.rs"]
mod recreated_tests;

fn fixture(name: &str) -> (PathBuf, query_annotation::QueryAnnotationFileFacts) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-argument-slot-write/src/{name}"
    ));
    let source = std::fs::read_to_string(&path).unwrap();
    let options = EmbeddedSqlOptions::configured("@app/db", &[]);
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(program, &source, &options)
    })
    .unwrap();
    (path, facts)
}

fn slot_write(facts: &query_annotation::QueryAnnotationFileFacts, name: &str) -> (u32, usize) {
    let Some(query_annotation::Expr::Function(function)) = facts.globals.get(name) else {
        panic!("missing function {name}");
    };
    let Some(index) = function.body.iter().find_map(|step| match step {
        query_annotation::Step::Bind(_, query_annotation::Expr::SlotWrite { index, .. }) => {
            Some(*index)
        }
        _ => None,
    }) else {
        panic!("missing recognized argument slot write in {name}");
    };
    (function.start, index)
}

#[path = "tests/collection.rs"]
mod collection;

#[path = "tests/replacement.rs"]
mod replacement;

#[path = "tests/sparse_aliases.rs"]
mod sparse_aliases;
