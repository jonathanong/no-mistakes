use crate::codebase::postgres::embedded::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use std::path::Path;

#[test]
fn relation_wrappers_over_cap_and_concat_trailing_edges() {
    let source = r#"
import { query } from "@example/db";
import sql from "sql-template-strings";

const BASE = (["entries", "audit_view"] satisfies readonly string[]) as const;
const WRAP_PAREN = (BASE);
const WRAP_SAT = BASE satisfies readonly string[];
const WRAP_CONST = BASE as const;
const EMPTY_ARR = [] as const;
const NUM_ARR = [1] as const;
const NOT_ARR = "entries" as const;
const AS_STR = ["entries"] as string[];
const ELEM_AS_STR = ["entries" as string] as const;
const BIG = ["a1","a2","a3","a4","a5","a6","a7","a8","a9","a10","a11","a12","a13","a14","a15","a16","a17"] as const;
const BIG_TUPLE: readonly ["a1","a2","a3","a4","a5","a6","a7","a8","a9","a10","a11","a12","a13","a14","a15","a16","a17"] = ["a1","a2","a3","a4","a5","a6","a7","a8","a9","a10","a11","a12","a13","a14","a15","a16","a17"];
type Five = "t1" | "t2" | "t3" | "t4" | "t5";
type Pair = "entries" | "records";
const STATIC_TBL = "entries";

export function wrapperAliases(
  a: (typeof WRAP_PAREN)[number],
  b: (typeof WRAP_SAT)[number],
  c: (typeof WRAP_CONST)[number],
  d: (typeof EMPTY_ARR)[number],
  e: (typeof NUM_ARR)[number],
  f: (typeof NOT_ARR)[number],
  g: (typeof AS_STR)[number],
  g2: (typeof ELEM_AS_STR)[number],
  h: (typeof BIG)[number],
  h2: (typeof BIG_TUPLE)[number],
  m: (typeof MISSING)[number],
  n: string[number],
  x: string,
) {
  query(`SELECT * FROM ${a}`);
  query(`SELECT * FROM ${b}`);
  query(`SELECT * FROM ${c}`);
  query(`SELECT * FROM ${g2}`);
  query(`SELECT * FROM ${h2}`);
  query(`SELECT * FROM ${d} ${e} ${f} ${g} ${h} ${m} ${n}`);
  return query(`SELECT * FROM ${STATIC_TBL} WHERE id = ${x}`);
}

export function concatAndAssignEdges(f1: Five, f2: Five, p: Pair, dyn: string, obj: any) {
  query(`SELECT * FROM ${f1} JOIN ${f2}`);
  query("SELECT * FROM " + f1 + " JOIN " + f2);
  query(dyn + ` SELECT * FROM ${p}`);
  query("UPDATE entries SET a = 1" + dyn + " RETURNING *" + " -- tail");
  query(dyn + " RETURNING *");

  let q: any = "SELECT * FROM entries";
  q = "SELECT id FROM narrow";
  q -= 1;
  q = obj + " ORDER BY id";
  q = q - 1;
  q += dyn;
  query(q);

  const c = sql`UPDATE entries SET a = 1`;
  query(c.append(" WHERE id = 1").append(" RETURNING *"));
  query(c?.append(" RETURNING *"));
  query(obj.other(" RETURNING *"));
  return query(sql`SELECT * FROM entries`.append(dyn));
}
"#;
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(Path::new("src/edges.ts"), source, &options);
    assert!(facts.calls.len() >= 14);
    let has_returning = facts
        .call_fragments
        .values()
        .flatten()
        .any(|f| f.sql.contains("RETURNING *"));
    assert!(has_returning);
    let has_entries = facts
        .call_fragments
        .values()
        .flatten()
        .any(|f| f.sql.contains("SELECT * FROM entries"));
    assert!(has_entries);
}

#[test]
fn logical_assign_inline_relation_append_copy_and_composed_return() {
    let source = r#"
import { query } from "@example/db";
import sql from "sql-template-strings";

type Pair = "entries" | "records";
type One = "entries";

function buildComposed(dyn: string) {
  let q = "SELECT id FROM narrow";
  q += dyn;
  q += " UNION ALL SELECT entries.* FROM entries";
  return q + " ORDER BY id";
}

function makeUpdate(flag: boolean) {
  if (flag) {
    const a = sql`UPDATE entries`;
    a.append(" AS t SET active = true");
    return a;
  }
  const b = sql`UPDATE records`;
  b.append(" AS t SET active = false");
  return b;
}

export function reviewEdgeCases(p: Pair, one: One, dyn: string, flag: boolean) {
  let logical = "SELECT * FROM entries";
  logical ||= "SELECT * FROM records";
  logical += dyn;
  query(logical);

  const inlinePair = sql`UPDATE `;
  query(
    inlinePair
      .append(p)
      .append(" AS t")
      .append(" SET active = true RETURNING t.*")
      .append(dyn),
  );

  const inlineSingle = sql`UPDATE `;
  query(inlineSingle.append(one).append(" SET active = true RETURNING *").append(dyn));

  const prefixed = sql`/* prefix */ `;
  prefixed.append(`UPDATE ${p}`);
  prefixed.append(" SET active = true RETURNING *");
  prefixed.append(dyn);
  query(prefixed);

  const copy = prefixed;
  query(copy);

  query(buildComposed(dyn));

  const returnedBuilder = makeUpdate(flag);
  returnedBuilder.append(dyn);
  returnedBuilder.append(" RETURNING t.*");
  query(returnedBuilder);
}
"#;
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts =
        extract_embedded_sql_from_source(Path::new("src/review_edges.ts"), source, &options);
    let frags: Vec<_> = facts.call_fragments.values().flatten().collect();
    assert!(frags
        .iter()
        .any(|f| f.sql.contains("SELECT * FROM entries")));
    assert!(frags
        .iter()
        .any(|f| f.sql.contains("SELECT * FROM records")));
    assert!(frags.iter().any(|f| {
        f.sql.contains("RETURNING t.*")
            && f.context_sql
                .as_deref()
                .is_some_and(|c| c.contains("UPDATE entries AS t"))
    }));
    assert!(frags.iter().any(|f| {
        f.sql.contains("RETURNING t.*")
            && f.context_sql
                .as_deref()
                .is_some_and(|c| c.contains("UPDATE records AS t"))
    }));
    let returning_records_count = frags
        .iter()
        .filter(|f| {
            f.sql.contains("RETURNING *")
                && f.context_sql
                    .as_deref()
                    .is_some_and(|c| c.contains("/* prefix */ UPDATE records"))
        })
        .count();
    assert_eq!(returning_records_count, 2);
    assert!(frags
        .iter()
        .any(|f| f.sql.contains("UNION ALL SELECT entries.* FROM entries")));
    assert!(frags.iter().any(|f| {
        f.sql.contains("RETURNING t.*")
            && f.context_sql
                .as_deref()
                .is_some_and(|c| c.contains("UPDATE entries AS t SET active = true"))
    }));
    assert!(frags.iter().any(|f| {
        f.sql.contains("RETURNING t.*")
            && f.context_sql
                .as_deref()
                .is_some_and(|c| c.contains("UPDATE records AS t SET active = false"))
    }));
    assert_eq!(facts.call_fragments.len(), 7);
}
