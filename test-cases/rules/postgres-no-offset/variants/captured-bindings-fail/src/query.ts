import sql from "sql-template-strings";
import { query } from "@example/db";
const flag = runtimeFlag;
const statement = sql`SELECT id FROM users`;
function runBuilder() {
  query(flag ? statement : sql`SELECT 1`);
}
statement.append(sql` OFFSET 1`);
runBuilder();
let text = "SELECT id FROM text_users";
function runText() {
  query(flag ? text : "SELECT 2");
}
text = "SELECT id FROM text_users OFFSET 2";
runText();
const primitive = "SELECT id FROM accounts";
function staticCapture() {
  query(flag ? primitive : "SELECT 3");
}
function local() {
  const own = sql`SELECT id FROM local_users`;
  query(flag ? own : sql`SELECT 4`);
}
query(flag ? statement : sql`SELECT 5`);
