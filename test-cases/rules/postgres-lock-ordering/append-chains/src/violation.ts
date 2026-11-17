import { query, sql } from "@example/db";

function scopeSql(ids: string[]) {
  return sql` WHERE id = ANY(${ids}::uuid[])`;
}

export function lockWork(ids: string[]) {
  return query(sql`SELECT id FROM work`.append(scopeSql(ids)).append(sql` FOR UPDATE`));
}
