import sql, { type SQLStatement } from "sql-template-strings";
import { query } from "@example/db";
const gate = runtimeFlag;
function choose(input: SQLStatement): boolean {
  if (runtimeFlag) input.append(sql` OFFSET 9`);
  return runtimeFlag;
}
const statement = sql`SELECT id FROM users`;
const alias = statement;
const unrelated = sql`SELECT id FROM accounts OFFSET 2`;
query(choose(statement) ? statement : sql`SELECT 1`);
query(statement);
query(alias);
function reject(input: SQLStatement): boolean {
  input.append(sql` LIMIT 1`);
  return false;
}
const rejected = sql`SELECT id FROM users OFFSET 8`;
query(reject(rejected) ? rejected : sql`SELECT 2`);
query(gate ? unrelated : sql`SELECT 3`);
const member = sql`SELECT id FROM member_users`;
const other = { append(input: SQLStatement): boolean {
  input.append(sql` OFFSET 12`);
  return runtimeFlag;
} };
query(other.append(member) ? member : sql`SELECT 4`);
