import sql, { SQLStatement } from "sql-template-strings";
import { query } from "@example/db";
const gate = runtimeFlag;
function withOffset(input: SQLStatement): SQLStatement {
  return input.append(sql` OFFSET 7`);
}

const called = sql("SELECT id FROM call_users");
const calledAlias = called;
query(gate ? called : sql`SELECT 1`);
withOffset(called);
query(gate ? calledAlias : sql`SELECT 1`);
const raw = sql.raw("SELECT id FROM raw_users");
const rawAlias = raw;
query(gate ? raw : sql`SELECT 2`);
withOffset(raw);
query(gate ? rawAlias : sql`SELECT 2`);
const joined = sql.join([sql`SELECT id`, sql` FROM join_users`]);
const joinedAlias = joined;
query(gate ? joined : sql`SELECT 3`);
withOffset(joined);
query(gate ? joinedAlias : sql`SELECT 3`);

const missing = withOffset();
query(gate ? missing : sql`SELECT 4`);
const spread = withOffset(...argumentsList);
query(gate ? spread : sql`SELECT 5`);
