import sql, { type SQLStatement } from "sql-template-strings";
import { query } from "@example/db";
const gate = runtimeFlag;
function withOffset(q: SQLStatement, owner: string): SQLStatement {
  const alias = q.append(sql` FROM users WHERE owner_id = ${owner}`);
  alias.append(sql` OFFSET 7`);
  return alias.append(sql` LIMIT 3`);
}
query(gate
  ? withOffset(sql`SELECT ${a}, ${b}, ${c}, ${d}, ${e}, ${f}, ${g}, ${h}, ${i}`, "first")
  : withOffset(sql`SELECT ${j}`, "second"));
query(sql`${withOffset(sql`SELECT id`, "nested")}`);
query(gate ? withOffset(sql`SELECT id`, "fluent").append(sql` FOR UPDATE`) : sql`SELECT 2`);
const existing = sql`SELECT id`;
query(gate ? withOffset(existing, "safe") : sql`SELECT 9`);
query(gate ? withOffset(existing, sideEffect()) : sql`SELECT 3`);
query(gate ? withOffset?.(sql`SELECT id`, "optional") : sql`SELECT 4`);
query(gate ? withOffset(sql`SELECT id`) : sql`SELECT 5`);
query(gate ? withOffset(unknownSql, "unknown") : sql`SELECT 6`);
query(gate ? withOffset("SELECT id" as unknown as SQLStatement, "cast") : sql`SELECT 7`);
query(gate ? withOffset(...argumentsList) : sql`SELECT 8`);
const plainString = "SELECT id";
query(gate ? withOffset(plainString, "plain") : sql`SELECT 10`);
const dynamicBuilder = sql`SELECT id ${sql.raw(runtimeClause)}`;
query(gate ? withOffset(dynamicBuilder, "dynamic") : sql`SELECT 11`);
// These satisfy the helper's arity, but its SQL input remains opaque.
query(gate ? withOffset(...argumentsList, "spread") : sql`SELECT 12`);
query(gate ? withOffset(sql`SELECT id ${sql.raw(runtimeClause)}`, "opaque") : sql`SELECT 13`);
