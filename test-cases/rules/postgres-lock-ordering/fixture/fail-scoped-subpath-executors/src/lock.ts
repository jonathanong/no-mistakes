import { openTransaction } from "@example/db";
import type { TransactionQuery } from "@example/db/types";

// The root import and the `@example/db/types` subpath import both count as the
// configured module, so both functions are scanned.
export async function moveOrder(orderId: string) {
  await using tx = await openTransaction();
  await tx(`SELECT * FROM orders WHERE id = ANY($1) FOR UPDATE`, [orderId]);
}

export async function lockAccounts(run: TransactionQuery, ids: string[]) {
  return run(`SELECT * FROM accounts WHERE id = ANY($1) FOR UPDATE`, [ids]);
}
