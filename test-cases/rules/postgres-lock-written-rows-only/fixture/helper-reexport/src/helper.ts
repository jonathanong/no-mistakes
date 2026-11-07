import type { TransactionQuery } from "@example/db";
export async function lockUser(query: TransactionQuery, id: string) {
  return query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [id]);
}
export async function lockConstant(query: TransactionQuery) {
  await query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [42]);
}
export async function lockUnknown(query: TransactionQuery, id: string) {
  await query(`SELECT 1 FROM users WHERE id = $1 FOR UPDATE`, [String(id)]);
}
