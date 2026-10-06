// Same factory and type names as @example/db. This file lives in another package.
import { openTransaction, type TxExecutor } from "../transaction";

export async function moveOrder(orderId: string) {
  await using tx = await openTransaction();
  await tx(`SELECT * FROM orders WHERE id = ANY($1) FOR UPDATE`, [orderId]);
}

export async function lockAccounts(run: TxExecutor, ids: string[]) {
  return run(`SELECT * FROM accounts WHERE id = ANY($1) FOR UPDATE`, [ids]);
}
