import { openTransaction } from "pg";
import type { TxExecutor } from "pg";

const registry = { users: "users", teams: "teams" };
export async function factory(ids: number[]) {
  const tx = await openTransaction();
  tx.query("START TRANSACTION");
  Promise.all(ids.map((id) => tx.query(`SELECT ${id}`)));
  tx.query(`SELECT * FROM ${registry[tableName]}`);
}
export function typed(tx: TxExecutor, ids: number[]) {
  tx.query("COMMIT");
  Promise.all(ids.map((id) => tx.query(`SELECT ${id}`)));
  tx.query(`SELECT * FROM ${registry[tableName]}`);
}
