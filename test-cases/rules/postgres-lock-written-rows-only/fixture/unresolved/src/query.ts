import { beginTransaction, query, type TransactionQuery } from "@example/db";
export async function unknownSql(sql: string) {
  const tx = await beginTransaction();
  await tx.query(sql);
}
export async function unknownMembership(tx: TransactionQuery) {
  await tx.query(`SELECT id FROM items FOR UPDATE`);
}
