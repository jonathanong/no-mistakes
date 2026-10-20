import { query } from "@example/db";
import { sql } from "unconfigured-sql";

export function load(filter: string) {
  return query(sql`SELECT * FROM t WHERE ${filter}`);
}
