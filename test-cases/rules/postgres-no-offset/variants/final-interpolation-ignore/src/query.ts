import type { SQLStatement } from "sql-template-strings";
import { sql, query } from "@example/db";
const gate = runtimeFlag;
/* A later interpolation can mutate the object already passed to the tag. */
const mutated = sql`id FROM users`;
if (gate) mutated.append(sql` WHERE id = 1`);
query(sql`SELECT ${mutated} LIMIT ${(mutated.append(sql` OFFSET 1`), 0)}`);
const escaped = sql`id FROM accounts`;
if (gate) escaped.append(sql` WHERE id = 2`);
query(sql`SELECT ${escaped} LIMIT ${unknownHelper(escaped)}`);
function addOffset(q: SQLStatement): SQLStatement {
  q.append(sql` OFFSET 3`);
  return q;
}
let summarized = gate ? sql`id FROM branches` : sql`id FROM branches WHERE active`;
query(sql`SELECT ${summarized} LIMIT ${(addOffset(summarized), 0)}`);
const unchanged = sql`id FROM documents`;
if (gate) unchanged.append(sql` WHERE id = 3`);
query(sql`SELECT ${unchanged} LIMIT ${0}`);
query(sql`SELECT id FROM fresh_users ${sql`WHERE active`.append(gate ? sql` AND enabled` : sql` AND verified`)}`);

/* Primitive value capture and fresh nested appends keep their old behavior. */
let primitive = gate ? "first" : "second";
query(sql`SELECT ${primitive} LIMIT ${(primitive = "SELECT 9 OFFSET 9", 0)}`);
const located = gate
  ? sql`id FROM pages WHERE id = ${branchFirst} OFFSET ${skip}`
  : sql`id FROM pages WHERE id = ${branchSecond}`;
query(sql`SELECT ${located} LIMIT ${limit}`);

/* A physical branch directive suppresses only its own OFFSET token. */
const suppressedBranch = gate
  // no-mistakes-disable-next-line postgres-no-offset
  ? sql`id FROM suppressed_pages WHERE active OFFSET ${skip}`
  : sql`id FROM suppressed_pages WHERE active`;
query(sql`SELECT ${suppressedBranch} LIMIT ${limit}`);
const suppressedMutation = sql`id FROM suppressed_users`;
if (gate) suppressedMutation.append(sql` WHERE active`);
// no-mistakes-disable-next-line postgres-no-offset
query(sql`SELECT ${suppressedMutation} LIMIT ${(suppressedMutation.append(sql` OFFSET 4`), 0)}`);
