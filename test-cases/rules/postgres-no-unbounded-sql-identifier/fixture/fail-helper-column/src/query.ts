import { query } from "@example/db";

function storage(name: string) {
  return { entityColumn: name, table: name };
}

export function load(
  name: string,
  currentUserId: string,
  ids: string[],
  pageSize: number,
) {
  const { table, entityColumn } = storage(name);
  return query(
    `SELECT ${entityColumn} FROM ${table} WHERE user_id = ${currentUserId} AND id = ANY(${ids}) LIMIT ${pageSize}`,
  );
}
