import { query } from "@example/db";

export function read(
  sql: string,
  currentUserId: string,
  cursor: string,
  limit: number,
) {
  query(`SELECT id FROM users WHERE active`);
  query("SELECT id FROM users WHERE active");
  query(sql);
  query(`SELECT id FROM users WHERE id = $1 LIMIT $2`, [currentUserId, limit]);
  return query(
    `SELECT id FROM users WHERE id = ${currentUserId} AND id > ${cursor} LIMIT ${limit}`,
  );
}

export function conditional(flag: boolean, uid: string, limit: number) {
  const t = flag ? "users" : "accounts";
  return query(`SELECT id FROM ${t} WHERE id = ${uid} LIMIT ${limit}`);
}
