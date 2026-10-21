use super::*;
use crate::codebase::postgres::embedded::EmbeddedSqlKind;
use std::path::PathBuf;

fn caller() -> super::super::EmbeddedSqlFileFacts {
    extract_from_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
        "../../test-cases/rules/postgres-required-predicates/variants/builder-text/src/caller.mts",
    ))
}

fn call_at(
    facts: &super::super::EmbeddedSqlFileFacts,
    line: u32,
) -> &crate::codebase::postgres::embedded::EmbeddedSqlCall {
    facts
        .calls
        .iter()
        .find(|call| call.line == line)
        .unwrap_or_else(|| panic!("missing executor call on line {line}"))
}

#[test]
fn builder_text_executor_arguments_recover_composed_sql_and_fail_closed() {
    let facts = caller();
    let straight = call_at(&facts, 8);
    assert_eq!(straight.kind, EmbeddedSqlKind::Composed);
    assert_eq!(straight.variants.len(), 0);
    let sql = straight.sql_text.as_deref().expect("recovered SQL");
    assert!(sql.contains("WITH picked AS"), "{sql}");
    assert!(sql.contains("view_active_plans"), "{sql}");
    assert!(sql.contains("sql_placeholder_"), "{sql}");

    let branching = call_at(&facts, 17);
    assert_eq!(branching.kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(branching.variants.len(), 2, "{branching:#?}");
    assert!(
        branching
            .variants
            .iter()
            .all(|variant| variant.sql_text.contains("WITH picked AS")),
        "{branching:#?}"
    );

    for line in [22, 28, 35, 41, 48, 55, 60, 65, 70, 75, 81] {
        let call = call_at(&facts, line);
        assert_eq!(call.kind, EmbeddedSqlKind::Dynamic, "line {line}");
        assert!(
            call.variants.is_empty(),
            "line {line} must not recover builder.text alternatives: {call:#?}"
        );
        assert_ne!(
            call.kind,
            EmbeddedSqlKind::Composed,
            "line {line} must not take the builder.text composed path: {call:#?}"
        );
    }
}
