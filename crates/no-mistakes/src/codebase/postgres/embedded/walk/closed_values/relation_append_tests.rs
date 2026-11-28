use crate::codebase::postgres::embedded::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use std::path::Path;

#[test]
fn parenthesized_tuple_concat_template_and_multi_relation_returning() {
    let source = r#"
import { query } from "@example/db";
import sql from "sql-template-strings";

const PAREN_TUPLE: (readonly ["entries", "audit_view"]) = ["entries", "audit_view"];
type Pair = "entries" | "records";
type Single = "entries";

export function fromParenTuple(table: (typeof PAREN_TUPLE)[number], suffix: string) {
  return query(`SELECT * FROM ${table}` + suffix);
}

export function multiRelationReturning(table: Pair) {
  const q = sql`UPDATE `;
  q.append(table);
  q.append(" AS t SET flag = true FROM wide_join AS j WHERE t.id = j.id");
  q.append(" RETURNING j.*");
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

export function conciseArrowHelper(suffix: string) {
  const make = () => sql`SELECT * FROM entries`.append(suffix);
  return query(make());
}
"#;
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(Path::new("src/multi.ts"), source, &options);
    assert_eq!(facts.calls.len(), 8);
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
            "SELECT * FROM entriessql_placeholder_1".to_string(),
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
            Some(
                "UPDATE entries AS t SET flag = true FROM wide_join AS j WHERE t.id = j.id"
                    .to_string()
            ),
            Some(
                "UPDATE records AS t SET flag = true FROM wide_join AS j WHERE t.id = j.id"
                    .to_string()
            ),
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
    let arrow_frags: Vec<String> = facts
        .call_fragments
        .get(&facts.call_spans[7])
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|f| f.sql)
        .collect();
    assert!(arrow_frags.iter().any(|s| s == "SELECT * FROM entries"));
}
