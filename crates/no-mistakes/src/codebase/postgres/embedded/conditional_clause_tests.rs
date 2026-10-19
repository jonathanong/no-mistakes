use super::{extract_embedded_sql_from_source, EmbeddedSqlCall, EmbeddedSqlOptions};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/postgres-facts/embedded")
        .join(name)
}

fn read_calls(name: &str) -> Vec<EmbeddedSqlCall> {
    let source = std::fs::read_to_string(fixture(name)).expect("fixture");
    extract_embedded_sql_from_source(
        &fixture(name),
        &source,
        &EmbeddedSqlOptions::configured("@example/db", &[]),
    )
    .calls
    .into_iter()
    .filter(|call| call.callee == "read")
    .collect()
}

fn sql_of(call: &EmbeddedSqlCall) -> Vec<String> {
    let mut sql: Vec<_> = call
        .variants
        .iter()
        .map(|variant| variant.sql_text.clone())
        .collect();
    sql.sort();
    sql
}

#[test]
fn ternary_literal_clauses_are_sql_alternatives() {
    let calls = read_calls("conditional-clause-union.ts");
    assert_eq!(calls.len(), 5);
    let pair = [
        "SELECT id FROM orders WHERE account_id = $1 ".to_string(),
        "SELECT id FROM orders WHERE account_id = $1 AND id > $2".to_string(),
    ];
    for call in &calls[..4] {
        assert!(!call.is_unanalyzable(), "{call:?}");
        assert!(call
            .variants
            .iter()
            .all(|variant| variant.sql_text.contains("account_id")));
        assert_eq!(sql_of(call), pair, "{call:?}");
    }
    // `cf` inside the call, not the module `extra` declaration.
    assert!(
        calls[3].variants.len() == 2 && calls[3].variants.iter().all(|variant| variant.line == 21),
        "{:?}",
        calls[3]
    );
    assert_eq!(
        sql_of(&calls[4]),
        vec![
            "SELECT id FROM orders WHERE account_id = $1 ".to_string(),
            "SELECT id FROM orders WHERE account_id = $1 AND id < $2".to_string(),
            "SELECT id FROM orders WHERE account_id = $1 AND id > $2".to_string(),
        ]
    );
}

#[test]
fn unknown_arms_abstain_instead_of_dropping_the_arm() {
    let known = "SELECT id FROM orders WHERE account_id = $1 AND id > $2";
    let calls = read_calls("conditional-clause-abstain.ts");
    assert_eq!(calls.len(), 10);
    for call in calls {
        assert!(call.is_unanalyzable(), "{call:?}");
        assert!(call.variants.is_empty(), "{call:?}");
        assert_ne!(call.sql_text.as_deref(), Some(known), "{call:?}");
    }
}

#[test]
fn variant_cap_keeps_sixteen_and_abstains_past_it() {
    let calls = read_calls("conditional-clause-cap.ts");
    assert_eq!(calls.len(), 2, "{calls:?}");
    assert_eq!(calls[0].variants.len(), 16, "{:?}", calls[0].variants);
    assert!(calls[0]
        .variants
        .iter()
        .all(|variant| variant.sql_text.contains("account_id")));
    assert!(calls[1].is_unanalyzable(), "{:?}", calls[1]);
    assert!(calls[1].variants.is_empty(), "{:?}", calls[1].variants);
}
