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
fn chained_append_init_and_branching_alt_prefix_extension() {
    let source = r#"import { query } from "@example/db";
import sql from "sql-template-strings";
type Pair = "entries" | "records";
export function check(table: Pair, flag: boolean, tail: string) {
  const q = sql`UPDATE entries`.append(" AS t SET active = true").append(tail);
  q.append(" RETURNING t.*");
  query(q);
  const b = sql`UPDATE `;
  b.append(table);
  b.append(flag ? " AS t SET active = true" : " AS t SET active = false");
  b.append(tail);
  b.append(" RETURNING t.*");
  query(b);
}"#;
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(Path::new("src/chained_alt.ts"), source, &options);
    let frags: Vec<_> = facts.call_fragments.values().flatten().collect();
    for expected in [
        "UPDATE entries AS t SET active = true",
        "UPDATE entries AS t SET active = false",
        "UPDATE records AS t SET active = true",
        "UPDATE records AS t SET active = false",
    ] {
        assert!(frags.iter().any(|f| {
            f.sql.contains("RETURNING t.*") && f.context_sql.as_deref() == Some(expected)
        }));
    }
    assert!(
        frags
            .iter()
            .filter(|f| {
                f.sql.contains("RETURNING t.*")
                    && f.context_sql.as_deref() == Some("UPDATE entries AS t SET active = true")
            })
            .count()
            >= 2
    );
}
