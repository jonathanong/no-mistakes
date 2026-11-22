import { beginTransaction } from "@example/db";

export async function differentAliases() {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts a JOIN accounts b ON a.parent_id = b.id FOR UPDATE OF a FOR SHARE OF b`);
  await query(`UPDATE accounts SET touched = true`);
}

export async function sameAliasUsesStrongerLock() {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts a WHERE a.id = 1 FOR SHARE OF a FOR UPDATE OF a`);
  await query(`UPDATE accounts SET touched = true WHERE id = 1`);
}
