import sql from "sql-template-strings";
import { query, openTransaction } from "@example/db";
import { openTransaction as relativeTransaction } from "./variants-executor-support";
import type { TxExecutor } from "@example/db";
const gate = runtimeFlag;
const statement = sql`SELECT id FROM users`;
const run = openTransaction();
run(statement);
query(gate ? statement : sql`SELECT 1`);
const pending = relativeTransaction();
pending(statement);
query(gate ? statement : sql`SELECT 2`);
client.query(statement);
query(gate ? statement : sql`SELECT 3`);
function typed(execute: TxExecutor) {
  const local = sql`SELECT id FROM local_users`;
  execute(local);
  query(gate ? local : sql`SELECT 4`);
}
