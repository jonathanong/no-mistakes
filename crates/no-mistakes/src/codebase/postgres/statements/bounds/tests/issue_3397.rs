use super::{query, SqlBoundKind};
use crate::codebase::postgres::extract_sql_statement_facts;

fn fixture(name: &str) -> &'static str {
    match name {
        "statements" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-cases/rules/postgres-bounded-statements/issue-3397/statements.sql"
        )),
        "recall" => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-cases/rules/postgres-bounded-statements/issue-3397/parser/recall.sql"
        )),
        _ => unreachable!(),
    }
}

fn rendered(sql: &str) -> Vec<String> {
    let extracted = extract_sql_statement_facts(sql);
    assert!(
        !extracted.parse_failed,
        "parse failed: {:?}",
        extracted.parse_failures
    );
    extracted
        .bounds
        .iter()
        .map(|fact| {
            let kind = match fact.kind {
                SqlBoundKind::Select => "select",
                SqlBoundKind::Update => "update",
                SqlBoundKind::Delete => "delete",
            };
            format!("{kind}: {}", query(&fact.query))
        })
        .collect()
}

#[test]
fn issue_3397_scalar_calls_keep_explicit_limits() {
    let lines = rendered(fixture("statements"));
    assert_eq!(
        lines,
        [
            "select: capped entries",
            "select: capped entries",
            "update: flows[id=#1] (capped flows[id=#1 status=value] flows[id=#0])",
            "update: flows[id=#1] (capped flows[id=#1 status=value] flows[id=#0])",
            "select: capped flows",
            "update: flows[id=value]",
            "select: capped entries flows[id=#0]",
            "select: capped entries[id=#1] flows",
            "update: flows[id=#1] (capped flows[id=#1 status=value] flows[id=#0])",
        ],
        "actual bounds: {lines:?}"
    );
}

#[test]
fn datetime_scalars_keep_a_limit_unknown_calls_do_not() {
    let sql = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test-cases/rules/postgres-bounded-statements/issue-3397-reopen/bounds.sql"
    ));
    let extracted = extract_sql_statement_facts(sql);
    assert!(!extracted.parse_failed, "{:?}", extracted.parse_failures);
    let capped: Vec<bool> = extracted
        .bounds
        .iter()
        .map(|fact| fact.query.capped)
        .collect();
    assert_eq!(
        capped,
        vec![
            true, true, true, true, true, true, true, true, true, true, true, true, true, false,
            false, false, false, false, false,
        ],
        "actual caps: {capped:?}"
    );
}

#[test]
fn issue_3397_unknown_calls_and_unlimited_scalars_stay_open() {
    let lines = rendered(fixture("recall"));
    assert_eq!(
        lines,
        [
            "select: entries",
            "select: entries opaque",
            "select: entries opaque",
            "select: flows opaque",
            "select: capped entries",
        ],
        "actual bounds: {lines:?}"
    );
}
