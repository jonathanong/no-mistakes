import { query } from "@example/db";
import sql from "sql-template-strings";

export function literalCall() {
  return query(sql("SELECT id FROM accounts"));
}
