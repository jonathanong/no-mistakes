import { beginTransaction, type TransactionQuery } from "@example/db";
import { sql } from "@example/sql";
import { maybeWriteAccount } from "@example/account-maintenance";

export async function conditionalQualifiedLock(id: number, enabled: boolean) {
  await using query = await beginTransaction();
  if (enabled) {
    await query(`SELECT 1 FROM public.accounts WHERE id = $1 FOR SHARE`, [id]);
    await query(`UPDATE accounts SET touched = true WHERE id = $1`, [id]);
  }
}

export async function conditionalDynamicRelation(table: string, id: number, enabled: boolean) {
  await using tx = await beginTransaction();
  if (enabled) {
    await tx.query(sql`SELECT 1 FROM ${table} WHERE id = $1 FOR SHARE`, [id]);
    await tx.query(sql`UPDATE ${table} SET touched = true WHERE id = $1`, [id]);
  }
}

export async function callerOwnedExecutor(query: TransactionQuery, id: number) {
  await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
  await query(`UPDATE accounts SET touched = true WHERE id = $1`, [id]);
}

export async function conditionalWriteWithoutTransactionProof(executor: TransactionQuery, id: number, enabled: boolean) {
  await using query = await beginTransaction();
  if (enabled) {
    await query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
    await executor(`UPDATE accounts SET touched = true WHERE id = $1`, [id]);
  }
}

export async function opaqueHelper(id: number) {
  await using tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM accounts WHERE id = $1 FOR SHARE`, [id]);
  await maybeWriteAccount(tx.query, id);
}

export async function readOnlyWithAmbiguousTarget() {
  await using query = await beginTransaction();
  await query(`SELECT 1 FROM accounts JOIN account_events ON accounts.id = account_events.account_id FOR SHARE`);
}
