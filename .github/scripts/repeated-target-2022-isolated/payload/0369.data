import sql, { type SQLStatement } from "sql-template-strings";
import { query } from "@example/db";
const gate = runtimeFlag;
function withOffset(branchInput: SQLStatement, label: string): SQLStatement {
  return branchInput.append(sql` OFFSET 7`);
}
let historical = sql`SELECT id FROM original_users`;
const historicalAlias = historical;
const historicalChain = historicalAlias;
const unaffected = sql`SELECT id FROM accounts`;
query(gate ? historicalChain : sql`SELECT 11`);
if (gate) historical = sql`SELECT id FROM replacement_users`;
query(withOffset(historical, "branch"));
query(historicalAlias);
query(historicalChain);
query(gate ? unaffected : sql`SELECT 12`);
let parent = sql`SELECT id FROM original_orders`;
const child = parent;
if (gate) parent = sql`SELECT id FROM replacement_orders`;
query(withOffset(child, "reverse"));
query(parent);
