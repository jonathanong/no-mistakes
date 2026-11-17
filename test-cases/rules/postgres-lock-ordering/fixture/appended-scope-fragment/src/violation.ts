import { query } from "@example/db";
import sql from "sql-template-strings";

function scopeSql(ids: string[]) {
  return sql` WHERE id = ANY(${ids}::uuid[])`;
}

export function lockSources(ids: string[]) {
  return query(sql`SELECT id FROM sources`.append(scopeSql(ids)).append(sql` FOR UPDATE`));
}
