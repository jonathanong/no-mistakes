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
  return query(`SELECT * FROM ${table} ` + suffix);
}

export function plusEqualsTemplateWithContext(table: Pair) {
  let q = "UPDATE ";
  q += `${table} SET active = true RETURNING *`;
  return query(q);
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

export function multiInitAndTemplateLeaf(table: Pair, suffix: string) {
  let q = `UPDATE ${table}`;
  q += " SET active = true RETURNING *";
  query(q);
  query("SELECT * FROM " + `${table} ` + suffix);
  return query("SELECT * FROM " + `${table}` + suffix);
}
"#;
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(Path::new("src/multi.ts"), source, &options);
    assert_eq!(facts.calls.len(), 12);
    let frags_of = |idx: usize| -> Vec<String> {
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
        frags_of(0),
        vec![
            "SELECT * FROM entries sql_placeholder_1".to_string(),
            "SELECT * FROM audit_view sql_placeholder_1".to_string(),
        ]
    );
    assert_eq!(
        frags_of(1),
        vec![
            "UPDATE ".to_string(),
            "entries SET active = true RETURNING *".to_string(),
            "records SET active = true RETURNING *".to_string(),
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
        returning_ctxs(2),
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
        returning_ctxs(3),
        vec![
            Some("UPDATE entries".to_string()),
            Some("UPDATE records".to_string()),
        ]
    );
    assert!(returning_ctxs(4)
        .iter()
        .any(|c| c.as_deref() == Some("UPDATE entries SET a = 1")));
    assert!(frags_of(5).is_empty());
    assert_eq!(returning_ctxs(6), vec![Some("UPDATE entries".to_string())]);
    assert_eq!(
        returning_ctxs(7),
        vec![
            Some("UPDATE entries".to_string()),
            Some("UPDATE records".to_string()),
        ]
    );
    assert!(frags_of(8).iter().any(|s| s == "SELECT * FROM entries"));
    assert_eq!(
        returning_ctxs(9),
        vec![
            Some("UPDATE entries".to_string()),
            Some("UPDATE records".to_string()),
        ]
    );
    assert_eq!(
        frags_of(10),
        vec![
            "SELECT * FROM entries sql_placeholder_1".to_string(),
            "SELECT * FROM records sql_placeholder_1".to_string(),
        ]
    );
    assert_eq!(
        frags_of(11),
        vec!["SELECT * FROM sql_placeholder_1".to_string()]
    );
}

#[test]
fn logical_assign_context_and_leading_concat_template() {
    let source = r#"import { query } from "@example/db";
type Pair = "entries" | "records";
export function check(table: Pair, tail: string) {
  let q = "UPDATE narrow SET flag = true";
  q &&= "UPDATE wide SET flag = true";
  q += " RETURNING *";
  query(q);
  let u = "UPDATE ";
  u += `${table} SET active = true RETURNING *` + tail;
  query(u);
}"#;
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts =
        extract_embedded_sql_from_source(Path::new("src/logical_concat.ts"), source, &options);
    let frags: Vec<_> = facts.call_fragments.values().flatten().collect();
    for (needle, ctx) in [
        ("RETURNING *", "UPDATE narrow SET flag = true"),
        ("RETURNING *", "UPDATE wide SET flag = true"),
        ("entries SET active = true RETURNING *", "UPDATE "),
        ("records SET active = true RETURNING *", "UPDATE "),
    ] {
        assert!(frags
            .iter()
            .any(|f| f.sql.contains(needle) && f.context_sql.as_deref() == Some(ctx)));
    }
}
