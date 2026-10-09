use super::dependencies;
use crate::codebase::postgres::{query_annotation, EmbeddedSqlOptions};
use std::path::PathBuf;

#[test]
fn saved_expression_profiles_keep_reads_distinct_from_write_only_identities() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-cases/rules/postgres-require-query-annotation/fixture/helper-tracing-callback-profile/reads.mts");
    let source = std::fs::read_to_string(&path).unwrap();
    let facts = crate::ast::with_program(&path, &source, |program, _| {
        query_annotation::collect(
            program,
            &source,
            &EmbeddedSqlOptions::configured("@app/db", &[]),
        )
    })
    .unwrap();
    let query_annotation::Expr::Function(function) = &facts.globals["profile"] else {
        panic!("saved profile summary");
    };
    let mut function = function.clone();
    assert!(
        !facts.unmodeled_calls.is_empty(),
        "saved omitted-call projection"
    );
    function.body.extend(
        facts
            .unmodeled_calls
            .iter()
            .cloned()
            .map(query_annotation::Step::Effect),
    );
    let reads = dependencies::names(&function);
    for name in [
        "source",
        "builder",
        "extra",
        "promised",
        "items",
        "consumer",
        "object",
        "discardedInput",
        "templateInput",
        "Constructor",
        "opaqueInput",
        "target",
        "slots",
        "key",
        "sql",
        "tagInput",
        "getSlots",
        "replacement",
    ] {
        assert!(reads.values.contains(name), "{name}");
    }
    for name in ["local", "reserved", "shadow", "ignoredCreation"] {
        assert!(!reads.values.contains(name), "uninvoked/local {name}");
    }
    assert!(
        !reads.identities.contains("slots"),
        "a real read overrides write-only identity projection"
    );
}
