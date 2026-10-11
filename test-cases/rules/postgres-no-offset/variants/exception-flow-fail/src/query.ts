import { query } from "@example/db";
import sql from "sql-template-strings";
const reset = runtimeFlag;
let statement = reset ? "SELECT id FROM users OFFSET 9" : "SELECT 1";
try {
  mayThrow();
  statement = reset ? "SELECT 2" : "SELECT 3";
} catch {
  query(statement);
  query(reset ? statement : "SELECT 4");
} finally {
  query(statement);
  const local = reset ? "SELECT 5" : "SELECT 6";
  query(local);
}
const escaped = sql`SELECT 7`;
external(escaped);
try {
  external(escaped);
} catch {
  query(escaped);
}
