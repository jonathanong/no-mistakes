import { query } from "@bench/db";
import sql from "sql-template-strings";
import { header } from "../shared/consumer.mjs";

export function runA() {
  const stmt = sql`/* ok */ SELECT 1`;
  const _header = header;
  return query(stmt);
}
