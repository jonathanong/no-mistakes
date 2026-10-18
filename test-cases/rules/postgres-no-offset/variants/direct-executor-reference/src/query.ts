import { openTransaction, read, write, type TxExecutor } from "@example/db";

// Judged calls named by the executor-adapter rule. `run(read, account)` is not
// here: passing the executor as a value is not a judged offset. An untyped
// `client.query` has no receiver provenance, so it is not a judged offset.
export function list(
  flag: boolean,
  other: boolean,
  client: { query: (sql: string) => unknown },
) {
  read("SELECT id FROM orders WHERE account_id = $1 OFFSET 1"); // findings: offset
  const exec = read;
  exec("SELECT id FROM orders WHERE account_id = $1 OFFSET 2"); // findings: offset
  (flag ? read : write)("SELECT id FROM orders WHERE account_id = $1 OFFSET 3"); // findings: offset
  (flag ? read : (other ? write : read))("SELECT id FROM orders WHERE account_id = $1 OFFSET 4"); // findings: offset
  client.query("SELECT id FROM orders WHERE account_id = $1 OFFSET 5");
}

export function typed(tx: TxExecutor) {
  tx.query("SELECT id FROM orders WHERE account_id = $1 OFFSET 6"); // findings: offset
  return tx("SELECT id FROM orders WHERE account_id = $1 OFFSET 7"); // findings: offset
}

export async function opened() {
  const tx = await openTransaction();
  return tx("SELECT id FROM orders WHERE account_id = $1 OFFSET 7"); // findings: offset
}
