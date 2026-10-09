use crate::codebase::postgres::{
    query_annotation::{self, Expr, Step},
    EmbeddedSqlOptions,
};
use std::{cell::Cell, path::PathBuf};
#[test]
fn parameter_position_construction_visits_each_saved_parameter_once() {
    let path=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-mapped-context/parameter-positions.cjs");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@app/db", &[]),
        )
    })
    .unwrap();
    for (label, count) in [("short", 8), ("growing", 64), ("duplicate", 3)] {
        let function = facts
            .roots
            .iter()
            .find_map(|step| match step {
                Step::Hoisted(name, Expr::Function(function)) if name == label => Some(function),
                _ => None,
            })
            .unwrap();
        let visits = Cell::new(0);
        let positions = super::collect(
            function
                .params
                .iter()
                .inspect(|_| visits.set(visits.get() + 1)),
        );
        assert_eq!(visits.get(), count);
        if label == "duplicate" {
            assert_eq!(positions.get("first"), Some(&2));
            assert_eq!(positions.get("second"), Some(&1));
        } else {
            assert_eq!(positions.len(), count);
        }
    }
}
