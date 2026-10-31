import { query } from "@example/db";

export function load(
  table: string,
  currentUserId: string,
  ids: string[],
  limit: number,
) {
  return query(
    `SELECT id FROM ${table} WHERE user_id = ${currentUserId} AND id = ANY(${ids}) LIMIT ${limit}`,
  );
}
