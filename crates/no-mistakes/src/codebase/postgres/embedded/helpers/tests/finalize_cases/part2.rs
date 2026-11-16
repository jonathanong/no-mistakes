use super::*;

#[test]
fn variant_only_value_hole_keeps_a_placeholder() {
    use super::super::extract;
    use crate::codebase::postgres::embedded::EmbeddedSqlKind;
    let facts = extract("singleton-append.ts");
    let pending = facts.pending_imported.first().expect("singleton pending");
    let mut call = pending.template.clone();
    call.sql_text = None;
    call.variants.truncate(1);
    call.variants[0].sql_text = format!("SELECT id FROM items WHERE id = {}", sentinel::encode(0));
    let mut calls = [call];
    let mut relative = [];
    let mut fragments = [];
    let mut variants = [];
    let sites = vec![pending.sites[0].1.clone()];
    let _ = finalize(
        Walked {
            calls: &mut calls,
            call_spans: &[pending.span],
            pending_calls: &mut relative,
            pending_spans: &BTreeMap::new(),
            fragments: &mut fragments,
            fragment_variants: &mut variants,
        },
        &sites,
        &crate::fx::FxHashMap::default(),
    );
    assert_eq!(calls[0].kind, EmbeddedSqlKind::Dynamic);
    assert!(calls[0].variants.is_empty());
    assert_eq!(
        calls[0].sql_text.as_deref(),
        Some("SELECT id FROM items WHERE id = sql_placeholder_1")
    );
}

#[test]
fn unresolved_fragment_hole_keeps_the_tail_without_a_placeholder() {
    let hole = sentinel::encode(0);
    let sites = [fragment_site()];
    let spaced = finalize_text(
        format!("/* q */{hole} SELECT id FROM k WHERE id = ANY($1::uuid[]) ORDER BY id LIMIT 3"),
        &sites,
    );
    assert_eq!(
        spaced.as_deref(),
        Some("/* q */ SELECT id FROM k WHERE id = ANY($1::uuid[]) ORDER BY id LIMIT 3")
    );
    let glued = finalize_text(format!("/* c */{hole}SELECT id FROM k"), &sites);
    assert_eq!(glued.as_deref(), Some("/* c */ SELECT id FROM k"));
    assert!(!glued.unwrap_or_default().contains("sql_placeholder_"));
}

#[test]
fn fragment_hole_beside_an_identifier_keeps_both_sides() {
    let fragment = sentinel::encode(0);
    let table = sentinel::encode(1);
    let sites = [fragment_site(), fragment_site()];
    let sql = finalize_text(
        format!(
            "WITH candidate AS (SELECT id FROM {table} t WHERE flag IS NULL{fragment} ORDER BY id FOR UPDATE SKIP LOCKED)"
        ),
        &sites,
    );
    assert_eq!(
        sql.as_deref(),
        Some(
            "WITH candidate AS (SELECT id FROM sql_placeholder_1 t WHERE flag IS NULL ORDER BY id FOR UPDATE SKIP LOCKED)"
        )
    );
}

#[test]
fn lock_table_hole_keeps_an_identifier_placeholder() {
    let hole = sentinel::encode(0);
    let sql = finalize_text(
        format!("/* lockReference */ LOCK TABLE {hole} IN ACCESS EXCLUSIVE MODE"),
        &[],
    );
    assert_eq!(
        sql.as_deref(),
        Some("/* lockReference */ LOCK TABLE sql_placeholder_1 IN ACCESS EXCLUSIVE MODE")
    );
}
