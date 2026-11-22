import { beginTransaction, query as poolQuery } from "@example/db";

export async function unknownReadOnlyTarget() {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts JOIN account_events ON accounts.id = account_events.account_id FOR SHARE`);
}

async function lockAccount(query: typeof poolQuery, id: number) {
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
}

export async function readOnlyHelper(id: number) {
  await using tx = await beginTransaction();
  await lockAccount(tx.query, id);
}
