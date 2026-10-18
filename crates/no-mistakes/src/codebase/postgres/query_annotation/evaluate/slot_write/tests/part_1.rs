use super::super::{Evaluator, File, Value};
use crate::codebase::postgres::query_annotation::modules::Module;
use crate::codebase::postgres::{
    extract_embedded_sql_from_program, query_annotation, EmbeddedSqlOptions,
};
use crate::codebase::ts_source::facts::TsFileFacts;
use crate::fx::{fx_map, FxHashMap};
use std::path::PathBuf;

#[path = "../recreated_tests.rs"]
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

#[test]
fn only_canonical_static_arguments_indexes_are_collected_as_slot_writes() {
    let (_, strict) = fixture("query.mts");
    let (_, sloppy) = fixture("sloppy.cjs");
    let (strict_start, strict_index) = slot_write(&strict, "strictReplacement");
    let (sloppy_start, sloppy_index) = slot_write(&sloppy, "sloppyReplacement");

    assert_eq!(strict_index, 0);
    assert_eq!(sloppy_index, 0);
    assert!(!strict.mapped_arguments.contains(&strict_start));
    assert!(sloppy.mapped_arguments.contains(&sloppy_start));

    let Some(query_annotation::Expr::Function(function)) = strict.globals.get("dynamicReplacement")
    else {
        panic!("missing dynamic key case");
    };
    assert!(matches!(
        function.body.iter().find_map(|step| match step {
            query_annotation::Step::Bind(_, expr) => Some(expr),
            _ => None,
        }),
        Some(query_annotation::Expr::OpaqueWrite { .. })
    ));
}

#[test]
fn argument_slot_index_classification_is_canonical_and_receiver_specific() {
    let (_, facts) = fixture("index-cases.mts");
    let Some(query_annotation::Expr::Function(function)) = facts.globals.get("indexCases") else {
        panic!("missing index classification fixture function");
    };
    let mut indexes = Vec::new();
    let mut opaque_writes = 0;
    for step in &function.body {
        let query_annotation::Step::Bind(_, expression) = step else {
            continue;
        };
        match expression {
            query_annotation::Expr::SlotWrite { index, .. } => indexes.push(*index),
            query_annotation::Expr::OpaqueWrite { .. } => opaque_writes += 1,
            _ => {}
        }
    }

    assert_eq!(indexes, [0, 1, 2, 0]);
    // Fractional, unsafe/huge, noncanonical, nonnumeric, and dynamic indexes
    // stay opaque. Other receivers are classified at runtime conservatively.
    assert_eq!(opaque_writes, 6);
}

