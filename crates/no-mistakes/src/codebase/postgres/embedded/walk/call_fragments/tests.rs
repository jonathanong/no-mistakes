use crate::codebase::postgres::embedded::{extract_embedded_sql_from_source, EmbeddedSqlOptions};
use std::path::Path;

#[test]
fn inline_append_relation_empty_args_and_closed_type_edges() {
    let source = r#"import { query } from "@example/db";
import sql from "sql-template-strings";
const SINGLE = ["wide"] as const;
const ALIAS_SINGLE = SINGLE;
type SingleTable = (typeof ALIAS_SINGLE)[number];
type Pair = "wide" | "narrow";
type BadNumType = 1;
const BAD_TUPLE_NUM: [1] = [1];
const BAD_TUPLE_OPT: [a?: "wide"] = [];
const BAD_TUPLE_STR: [string] = ["wide"];
const BAD_TUPLE_REST: [...string[]] = [];
const BAD_CONST_ARR = [1] as const;
type FromBadNum = (typeof BAD_TUPLE_NUM)[number];
type FromBadOpt = (typeof BAD_TUPLE_OPT)[number];
type FromBadStr = (typeof BAD_TUPLE_STR)[number];
type FromBadRest = (typeof BAD_TUPLE_REST)[number];
type FromBadArr = (typeof BAD_CONST_ARR)[number];
type FromQualified = (typeof ns.TABLES)[number];

export function run(
  single: SingleTable,
  pair: Pair,
  numUnion: 1 | 2,
  badNum: FromBadNum,
  badOpt: FromBadOpt,
  badStr: FromBadStr,
  badRest: FromBadRest,
  badArr: FromBadArr,
  badQual: FromQualified,
  dynTail: string,
  qParam: any,
  memberObj: { table: SingleTable; direct: (typeof SINGLE)[number] },
) {
  const q1 = sql`SELECT * FROM `;
  query(q1.append().append(single).append(dynTail));

  const q2 = sql`SELECT * FROM `;
  q2.append(sql`wide`);
  query(q2.append(dynTail));

  query(qParam.append(sql`SELECT * FROM wide`));
  query(unknownBuilder.append(sql`SELECT * FROM wide`));

  let q3 = `UPDATE ${pair} SET a = 1`;
  q3 += " WHERE id = 1";
  q3 += " RETURNING *";
  q3 += dynTail;
  query(q3);

  query(`SELECT id FROM wide WHERE note = '${memberObj.table}' AND other = '${memberObj.direct}'`);
  query(`SELECT * FROM ${numUnion}`);
  query(`SELECT * FROM ${badNum}`);
  query(`SELECT * FROM ${badOpt}`);
  query(`SELECT * FROM ${badStr}`);
  query(`SELECT * FROM ${badRest}`);
  query(`SELECT * FROM ${badArr}`);
  query(`SELECT * FROM ${badQual}`);
}"#;
    let options = EmbeddedSqlOptions::configured("@example/db", &[]);
    let facts = extract_embedded_sql_from_source(Path::new("src/edge.ts"), source, &options);
    let frags: Vec<_> = facts.call_fragments.values().flatten().collect();
    assert!(frags.iter().any(|f| f.sql.contains("SELECT * FROM wide")));
    assert!(frags.iter().any(|f| f.sql.contains("RETURNING *")
        && f.context_sql.as_deref() == Some("UPDATE wide SET a = 1 WHERE id = 1")));
    assert!(frags.iter().any(|f| f.sql.contains("RETURNING *")
        && f.context_sql.as_deref() == Some("UPDATE narrow SET a = 1 WHERE id = 1")));
    assert!(facts.calls.iter().all(|c| c
        .sql_text
        .as_deref()
        .is_none_or(|s| !s.contains("note = 'wide'"))));
}
