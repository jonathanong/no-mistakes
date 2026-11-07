use super::*;
use crate::codebase::postgres::embedded::{EmbeddedSqlFileFacts, EmbeddedSqlKind};
use crate::codebase::postgres::statements::extract_sql_statement_facts_for_embedded_call;

fn extract_view(name: &str) -> EmbeddedSqlFileFacts {
    let path = crate::test_support::rule_fixture_root("postgres-no-transformed-uuid-predicates")
        .join(name);
    extract_embedded_sql_from_source(
        &path,
        &std::fs::read_to_string(&path).expect("saved fixture"),
        &EmbeddedSqlOptions::configured("db", &["read".into()]),
    )
}

#[test]
fn direct_builder_text_preserves_the_view_probe_and_appended_bind() {
    let file = extract_view("append-view-object-pass.ts");
    let paired = extract_view("append-view-text-pass.ts");
    assert_eq!(file.calls, paired.calls);
    let [call] = file.calls.as_slice() else {
        panic!("expected one executor call: {file:#?}");
    };
    assert_eq!(call.kind, EmbeddedSqlKind::Composed);
    assert_eq!(call.variants.len(), 1);
    assert!(!call.variants[0].append_sites.is_empty());
    let sql = call.sql_text.as_deref().expect("complete composed SQL");
    assert!(sql.contains("JOIN plain_view v ON v.id = COALESCE(t.parent_id, t.id)"));
    assert!(sql.contains("WHERE t.id = sql_placeholder_"));
    assert_eq!(call.recovered_placeholder_positions.len(), 1);
    let facts = extract_sql_statement_facts_for_embedded_call(call).expect("statement facts");
    assert!(!facts.parse_failed, "{facts:#?}");
    assert_eq!(facts.transformed_column_uses.len(), 2);
    for fact in facts.transformed_column_uses {
        assert_eq!(fact.table, "a");
        let probe = fact.driving_probe.expect("view probe retained");
        assert_eq!(probe.table, "plain_view");
        assert_eq!(probe.column, "id");
    }
}

#[test]
fn direct_builder_text_keeps_transforms_on_the_probed_view_column() {
    let file = extract_view("append-view-object-positive.ts");
    let paired = extract_view("append-view-text-positive.ts");
    assert_eq!(file.calls, paired.calls);
    let [call] = file.calls.as_slice() else {
        panic!("expected one executor call: {file:#?}");
    };
    let facts = extract_sql_statement_facts_for_embedded_call(call).expect("statement facts");
    assert!(!facts.parse_failed, "{facts:#?}");
    assert_eq!(facts.transformed_column_uses.len(), 2);
    for fact in facts.transformed_column_uses {
        assert_eq!(fact.table, "plain_view");
        assert!(fact.driving_probe.is_none());
    }
}

#[test]
fn direct_builder_text_rejects_unproven_owners_arguments_and_mutations() {
    let file = extract_view("append-view-controls.ts");
    assert_eq!(file.calls.len(), 5, "{file:#?}");
    for call in file.calls {
        assert_eq!(call.kind, EmbeddedSqlKind::Dynamic, "{call:#?}");
        assert!(call.variants.is_empty(), "{call:#?}");
        assert!(call.sql_text.is_none(), "{call:#?}");
    }
}

#[test]
fn direct_builder_object_keeps_alias_ownership_and_non_appended_classification() {
    let file = extract_view("append-view-object-controls.ts");
    let [alias, untouched, conditional] = file.calls.as_slice() else {
        panic!("expected three executor calls: {file:#?}");
    };
    assert_eq!(alias.kind, EmbeddedSqlKind::Dynamic);
    assert!(alias.variants.is_empty());
    assert_eq!(untouched.kind, EmbeddedSqlKind::ImmutableLocal);
    assert!(untouched.variants.is_empty());
    assert_eq!(conditional.kind, EmbeddedSqlKind::Dynamic);
    assert_eq!(
        conditional.sql_text.as_deref(),
        Some("SELECT id FROM threads")
    );
    assert_eq!(conditional.variants.len(), 2);
}
