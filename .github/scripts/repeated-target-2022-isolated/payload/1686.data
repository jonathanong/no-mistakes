import type { SQLStatement } from "sql-template-strings";
import { sql, query } from "@example/db";
const gate = runtimeFlag;

/* The configured tag splices mutable fragments after every expression runs. */
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

/* A final ordinary bind and an append on a fresh fragment remain recoverable. */
const unchanged = sql`id FROM documents`;
if (gate) unchanged.append(sql` WHERE id = 3`);
query(sql`SELECT ${unchanged} LIMIT ${0}`);
query(sql`SELECT id FROM fresh_users ${sql`WHERE active`.append(gate ? sql` AND enabled` : sql` AND verified`)}`);

/* Primitive interpolations are copied before their binding is reassigned. */
let primitive = gate ? "first" : "second";
query(sql`SELECT ${primitive} LIMIT ${(primitive = "SELECT 9 OFFSET 9", 0)}`);
query(sql`SELECT ${"copied"} LIMIT ${0}`);

/* With no prior reference, the final expression describes the returned SQL. */
const single = sql`id FROM single_users`;
query(sql`SELECT ${single.append(sql` WHERE active LIMIT 1`)}`);
const helperOnly = sql`id FROM helper_users`;
query(sql`SELECT ${addOffset(helperOnly)}`);

const located = gate
  ? sql`id FROM pages WHERE id = ${branchFirst} OFFSET ${skip}`
  : sql`id FROM pages WHERE id = ${branchSecond}`;
query(sql`SELECT ${located} LIMIT ${limit}`);

/* Unknown ordinary bind values never become fragment object references. */
const ordinary = runtimeValue;
query(sql`SELECT ${ordinary}, ${runtimeOwner} LIMIT ${0}`);
