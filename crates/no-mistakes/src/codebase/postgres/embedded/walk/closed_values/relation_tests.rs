use crate::codebase::postgres::embedded::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use std::path::Path;

#[test]
fn closed_relation_tuple_and_widened_array_types() {
    let source = r#"
import { query } from "@example/db";
const TUPLE: readonly ["entries", "audit_view"] = ["entries", "audit_view"];
const NAMED_TUPLE: [first: "records", second: "audit_view"] = ["records", "audit_view"];
const WIDENED: string[] = ["entries", "audit_view"];
const BAD_TUPLE: readonly ["entries", 1] = ["entries", 1];

export function fromTuple(table: (typeof TUPLE)[number], id: string) {
  return query(`SELECT * FROM ${table} WHERE id = $1`, [id]);
}
export function fromNamedTuple(table: (typeof NAMED_TUPLE)[number], id: string) {
  return query(`SELECT * FROM ${table} WHERE id = $1`, [id]);
}
export function fromWidened(table: (typeof WIDENED)[number], id: string) {
  return query(`SELECT * FROM ${table} WHERE id = $1`, [id]);
}
export function fromBadTuple(table: (typeof BAD_TUPLE)[number], id: string) {
  return query(`SELECT * FROM ${table} WHERE id = $1`, [id]);
}
"#;
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(Path::new("src/tuples.ts"), source, &options);
    assert_eq!(facts.calls.len(), 4);
    let frags = |idx: usize| -> Vec<String> {
        facts
            .call_fragments
            .get(&facts.call_spans[idx])
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .map(|f| f.sql)
            .collect()
    };
    assert_eq!(
        frags(0),
        vec![
            "SELECT * FROM entries WHERE id = $1",
            "SELECT * FROM audit_view WHERE id = $1",
        ]
    );
    assert_eq!(
        frags(1),
        vec![
            "SELECT * FROM records WHERE id = $1",
            "SELECT * FROM audit_view WHERE id = $1",
        ]
    );
    assert_eq!(
        frags(2),
        vec!["SELECT * FROM sql_placeholder_1 WHERE id = $1"]
    );
    assert_eq!(
        frags(3),
        vec!["SELECT * FROM sql_placeholder_1 WHERE id = $1"]
    );
}
