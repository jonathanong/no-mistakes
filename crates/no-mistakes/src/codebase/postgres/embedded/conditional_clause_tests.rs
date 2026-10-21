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
fn logical_clauses_are_alternatives_until_an_unknown_branch() {
    let calls = read_calls("conditional-clause-logical.ts");
    assert_eq!(calls.len(), 16, "{calls:?}");
    let greater = "SELECT id FROM orders WHERE account_id = $1 AND id > $2";
    let lesser = "SELECT id FROM orders WHERE account_id = $1 AND id < $2";
    let empty = "SELECT id FROM orders WHERE account_id = $1 ";
    let pair = [lesser.to_string(), greater.to_string()];
    assert_eq!(sql_of(&calls[0]), pair, "or {calls:?}");
    assert_eq!(
        sql_of(&calls[1]),
        vec![empty.to_string(), lesser.to_string()],
        "and {calls:?}"
    );
    assert_eq!(sql_of(&calls[2]), pair, "inline {calls:?}");
    assert_eq!(
        sql_of(&calls[3]),
        vec![
            "SELECT id FROM orders WHERE account_id = $1 AND id < $2 /*no*/".to_string(),
            "SELECT id FROM orders WHERE account_id = $1 AND id < $2 /*yes*/".to_string(),
            "SELECT id FROM orders WHERE account_id = $1 AND id > $2 /*no*/".to_string(),
            "SELECT id FROM orders WHERE account_id = $1 AND id > $2 /*yes*/".to_string(),
        ],
        "independent holes {calls:?}"
    );
    assert_eq!(
        sql_of(&calls[4]),
        vec![empty.to_string(), lesser.to_string(), greater.to_string()],
        "nested {calls:?}"
    );
    let kept = vec![empty.to_string(), greater.to_string()];
    assert_eq!(sql_of(&calls[5]), kept, "coalesce {calls:?}");
    assert_eq!(sql_of(&calls[6]), kept, "coalesce fallback {calls:?}");
    assert_eq!(
        sql_of(&calls[7]),
        vec![empty.to_string()],
        "falsy and {calls:?}"
    );
    assert_eq!(
        sql_of(&calls[8]),
        vec![greater.to_string()],
        "truthy or {calls:?}"
    );
    for call in &calls[9..] {
        assert!(call.is_unanalyzable(), "{call:?}");
        assert!(call.variants.is_empty(), "{call:?}");
        assert_ne!(call.sql_text.as_deref(), Some(greater), "{call:?}");
    }
}

#[test]
fn identifier_holes_follow_same_file_helpers_until_a_failed_call() {
    let calls = read_calls("identifier-clause-calls.ts");
    assert_eq!(calls.len(), 6, "{calls:?}");
    assert_eq!(
        sql_of(&calls[0]),
        vec![
            "SELECT id FROM accounts WHERE id = $1".to_string(),
            "SELECT id FROM orders WHERE id = $1".to_string(),
        ],
        "nested {calls:?}"
    );
    assert_eq!(
        sql_of(&calls[1]),
        vec!["SELECT id FROM orders WHERE account_id = $1".to_string()],
        "duplicate {calls:?}"
    );
    // Failed nested call, depth cap, NUL text, and a statement used as an identifier.
    for call in &calls[2..] {
        assert!(call.is_unanalyzable(), "{call:?}");
        assert!(call.variants.is_empty(), "{call:?}");
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
