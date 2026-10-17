import type { SQLStatement } from "sql-template-strings";
import { sql, query } from "@example/db";
const gate = runtimeFlag;
function addOffset(q: SQLStatement): SQLStatement {
  q.append(sql` OFFSET 3`);
  return q;
}

/* Assignment returns the old builder reference to the enclosing consumer. */
const direct = gate ? sql`id FROM direct_users` : sql`id FROM direct_users WHERE active`;
let other = 0;
query(sql`SELECT ${direct} LIMIT ${((other = direct).append(sql` OFFSET 1`), 0)}`);
const computed = sql`id FROM computed_users`;
query(sql`SELECT ${computed} LIMIT ${((other = computed)["append"](sql` OFFSET 2`), 0)}`);
const summarized = gate ? sql`id FROM helper_users` : sql`id FROM helper_users WHERE active`;
query(sql`SELECT ${summarized} LIMIT ${(addOffset(other = summarized), 0)}`);
const wrapped = gate ? sql`id FROM wrapped_users` : sql`id FROM wrapped_users WHERE active`;
query(sql`SELECT ${wrapped} LIMIT ${(((other as unknown) = wrapped).append(sql` OFFSET 4`), 0)}`);
const nested = gate ? sql`id FROM nested_users` : sql`id FROM nested_users WHERE active`;
let nestedAlias = 0;
query(sql`SELECT ${nested} LIMIT ${((other = (nestedAlias = nested))["append"](sql` OFFSET 5`), 0)}`);
const escaped = gate ? sql`id FROM escaped_users` : sql`id FROM escaped_users WHERE active`;
query(sql`SELECT ${escaped} LIMIT ${(unknownHelper(other = escaped), 0)}`);

/* A pure assignment result used only as a discarded value is harmless. */
const unchanged = gate
  ? sql`id FROM unchanged_users WHERE id = ${firstOwner} OFFSET ${skip}`
  : sql`id FROM unchanged_users WHERE id = ${secondOwner}`;
query(sql`SELECT ${unchanged} LIMIT ${(other = unchanged, limit)}`);
query(sql`SELECT ${other}`);

/* Compound string assignment keeps its result as a primitive bind. */
let primitive = "copied";
const primitiveCopy = (primitive += " value");
query(sql`SELECT ${primitiveCopy} LIMIT ${0}`);
