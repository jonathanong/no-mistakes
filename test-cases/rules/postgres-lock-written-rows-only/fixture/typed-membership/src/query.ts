import { type TransactionQuery } from "@example/db";
export async function lock(tx: TransactionQuery) {
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
}
