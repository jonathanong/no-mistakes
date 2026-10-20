import sql from "sql-template-strings";
import { query } from "@example/db";

const statement = sql`SELECT id FROM ` + "topics";

export function load() {
  return query(statement);
}
