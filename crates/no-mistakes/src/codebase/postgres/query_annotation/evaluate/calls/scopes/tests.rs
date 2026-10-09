use super::shadow_names;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use std::path::PathBuf;

#[test]
fn saved_growing_shadow_indexes_do_linear_construction_and_lookup_work() {
    for size in [32, 128] {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-shadow-index/capture-{size}.cjs"));
        let source = std::fs::read_to_string(&path).unwrap();
        let facts = crate::ast::with_program(&path, &source, |program, _| {
            query_annotation::collect(
                program,
                &source,
                &EmbeddedSqlOptions::configured("@app/db", &[]),
            )
        })
        .unwrap();
        let query_annotation::Expr::Function(function) = &facts.globals["probe"] else {
            panic!("saved function summary");
        };
        let shadows = shadow_names(function);
        for name in ["self", "arguments", "bare", "hoisted"] {
            assert!(shadows.contains(name), "{name}");
        }
        for index in 0..size {
            assert!(shadows.contains(&format!("parameter{index}")));
            assert!(shadows.contains(&format!("local{index}")));
        }
        let mut capture_lookups = 0;
        for index in 0..size {
            capture_lookups += 1;
            assert!(!shadows.contains(&format!("capture{index}")));
        }
        assert_eq!(capture_lookups, size);
        assert!(
            shadows.construction_work <= 4 * size + 20,
            "one body/parameter pass, not one per captured binding"
        );
        assert!(shadows.construction_work + capture_lookups <= 5 * size + 20);
    }
}
