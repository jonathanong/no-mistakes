import { openTransaction } from "@example/db";
import type { TransactionQuery } from "@example/db/types";

// Ordered locks, so the lock-ordering rule itself reports nothing here.
export async function moveOrder(orderId: string) {
  await using tx = await openTransaction();
  await tx(`SELECT * FROM orders WHERE id = ANY($1) ORDER BY id FOR UPDATE`, [orderId]);
}

export async function lockAccounts(run: TransactionQuery, ids: string[]) {
  return run(`SELECT * FROM accounts WHERE id = ANY($1) ORDER BY id FOR UPDATE`, [ids]);
}
