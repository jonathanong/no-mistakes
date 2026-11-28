use crate::codebase::postgres::embedded::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use std::path::Path;

#[test]
fn closed_relation_tuple_and_widened_array_types() {
    let source = r#"
import { query } from "@example/db";
const TUPLE: readonly ["entries", "audit_view"] = ["entries", "audit_view"];
const NAMED_TUPLE: [first: "records", second: "audit_view"] = ["records", "audit_view"];
const OPT_TUPLE: [first?: "entries"] = [];
const WIDENED: string[] = ["entries", "audit_view"];
const BAD_TUPLE: readonly ["entries", 1] = ["entries", 1];

export function fromTuple(table: (typeof TUPLE)[number], id: string) {
  return query(`SELECT * FROM ${table} WHERE id = $1`, [id]);
}
export function fromNamedTuple(table: (typeof NAMED_TUPLE)[number], id: string) {
  return query(`SELECT * FROM ${table} WHERE id = $1`, [id]);
}
export function fromOptTuple(table: (typeof OPT_TUPLE)[number], id: string) {
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
    assert_eq!(facts.calls.len(), 5);
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
    assert_eq!(
        frags(4),
        vec!["SELECT * FROM sql_placeholder_1 WHERE id = $1"]
    );
}

#[test]
fn alias_const_array_parameterized_tag_and_reassigned_helper() {
    let source = r#"
import { query } from "@example/db";
import sql from "sql-template-strings";

const NAMES = ["entries", "audit_view"] as const;
const ALIAS = NAMES;
const WIDENED_ALIAS = NAMES as readonly string[];

export function fromAliasedArray(table: (typeof ALIAS)[number], id: string) {
  return query(`SELECT * FROM ${table} WHERE id = $1`, [id]);
}

export function fromWidenedAlias(table: (typeof WIDENED_ALIAS)[number], id: string) {
  return query(`SELECT * FROM ${table} WHERE id = $1`, [id]);
}

export function parameterizedStarValue(filter: any) {
  const q = sql`SELECT ${"*"} FROM entries`;
  if (filter) q.append(filter);
  q.append(sql` UNION ALL SELECT ${"*"} FROM entries`);
  return query(q);
}

export function reassignedHelper(other: () => any) {
  let make = () => {
    const q = sql`SELECT * FROM entries`;
    return q;
  };
  make = other;
  return query(make());
}

export function destructuredVarBuilder(obj: { q: any }) {
  var q = sql`SELECT * FROM entries`;
  var { q } = obj;
  return query(q);
}

export function selfAdditionPreservesEarlierWildcard(filter: string) {
  let q = "SELECT * FROM entries";
  q = q + filter + " ORDER BY id";
  return query(q);
}

export function destructuredAssignClearsBuilder(obj: { q: any }) {
  let q = sql`SELECT * FROM entries`;
  ({ q } = obj);
  return query(q);
}
"#;
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(Path::new("src/aliases.ts"), source, &options);
    assert_eq!(facts.calls.len(), 7);
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
        vec!["SELECT * FROM sql_placeholder_1 WHERE id = $1"]
    );
    assert_eq!(
        frags(2),
        vec![
            "SELECT $1 FROM entries",
            " UNION ALL SELECT sql_placeholder_1 FROM entries",
        ]
    );
    assert!(frags(3).is_empty());
    assert!(frags(4).is_empty());
    assert!(frags(5).iter().any(|sql| sql == "SELECT * FROM entries"));
    assert!(frags(6).is_empty());
}

#[test]
fn parenthesized_tuple_concat_template_and_multi_relation_returning() {
    let source = r#"
import { query } from "@example/db";
import sql from "sql-template-strings";

const PAREN_TUPLE: (readonly ["entries", "audit_view"]) = ["entries", "audit_view"];
type Pair = "entries" | "records";

export function fromParenTuple(table: (typeof PAREN_TUPLE)[number], suffix: string) {
  return query(`SELECT * FROM ${table}` + suffix);
}

export function multiRelationReturning(table: Pair) {
  const q = sql`UPDATE `;
  q.append(table);
  q.append(" SET active = true RETURNING *");
  return query(q);
}

export function plusEqualsRelationReturning(table: Pair) {
  let q = "UPDATE ";
  q += table;
  q += " SET active = true RETURNING *";
  return query(q);
}

export function inlineAppendToExecutor(suffix: string) {
  const q = sql`UPDATE entries SET a = 1`;
  return query(q.append(" RETURNING * " + suffix));
}

export function destructuringDefaultAfterTarget() {
  let q = "SELECT * FROM entries";
  let result;
  [q, result = query(q)] = ["SELECT id FROM narrow", undefined];
  return result;
}

type Single = "entries";
export function singleAndBranchedRelationAppends(one: Single, pair: Pair, cond: boolean) {
  const a = sql`UPDATE `;
  a.append(one);
  a.append(" SET x = 1 RETURNING *");
  query(a);
  const b = sql`UPDATE `;
  if (cond) b.append(pair);
  b.append(" SET x = 1 RETURNING *");
  return query(b);
}
"#;
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(Path::new("src/multi.ts"), source, &options);
    assert_eq!(facts.calls.len(), 7);
    let first: Vec<String> = facts
        .call_fragments
        .get(&facts.call_spans[0])
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|f| f.sql)
        .collect();
    assert_eq!(
        first,
        vec![
            "SELECT * FROM entries__nm_placeholder__"
                .replace("__nm_placeholder__", "sql_placeholder_1"),
            "SELECT * FROM audit_viewsql_placeholder_1".to_string(),
        ]
    );
    let returning_ctxs = |idx: usize| -> Vec<Option<String>> {
        facts
            .call_fragments
            .get(&facts.call_spans[idx])
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|f| f.sql.contains("RETURNING"))
            .map(|f| f.context_sql)
            .collect()
    };
    assert_eq!(
        returning_ctxs(1),
        vec![
            Some("UPDATE entries".to_string()),
            Some("UPDATE records".to_string()),
        ]
    );
    assert_eq!(
        returning_ctxs(2),
        vec![
            Some("UPDATE entries".to_string()),
            Some("UPDATE records".to_string()),
        ]
    );
    assert!(returning_ctxs(3)
        .iter()
        .any(|c| c.as_deref() == Some("UPDATE entries SET a = 1")));
    assert!(facts
        .call_fragments
        .get(&facts.call_spans[4])
        .cloned()
        .unwrap_or_default()
        .is_empty());
    assert_eq!(returning_ctxs(5), vec![Some("UPDATE entries".to_string())]);
    assert_eq!(
        returning_ctxs(6),
        vec![
            Some("UPDATE entries".to_string()),
            Some("UPDATE records".to_string()),
        ]
    );
}
