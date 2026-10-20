import sql from "sql-template-strings";
import { query } from "@example/db";

const text = sql`SELECT ${1}`;
text.append(" AS sql_placeholder_1");

export function load() {
  return query(text);
}
