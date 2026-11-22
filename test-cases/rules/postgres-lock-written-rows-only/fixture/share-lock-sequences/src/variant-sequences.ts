import { beginTransaction } from "@example/db";

export async function variantWrite(id: number, enabled: boolean) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
  await query(enabled ? `UPDATE accounts SET touched = true WHERE id = $1` : `SELECT 1 FROM accounts WHERE id = $1`, [id]);
}

export async function variantLock(id: number, enabled: boolean) {
  await using query = await beginTransaction();
  await query(enabled ? `SELECT 1 FROM accounts WHERE id = $1 FOR SHARE` : `SELECT 1 FROM accounts WHERE id = $1`, [id]);
  await query(`UPDATE accounts SET touched = true WHERE id = $1`, [id]);
}

export async function unrelatedVariantWrite(id: number, enabled: boolean) {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
  await query(enabled ? `UPDATE account_events SET touched = true WHERE id = $1` : `SELECT 1 FROM accounts WHERE id = $1`, [id]);
}
