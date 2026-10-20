import sql from "sql-template-strings";
import { query } from "@example/db";

const statement = "SELECT id FROM topics".append(sql` AND active`);

export function load() {
  return query(statement);
}
