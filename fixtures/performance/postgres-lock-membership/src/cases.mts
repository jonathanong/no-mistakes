import type { PoolQuery, TransactionQuery } from "@example/db";
import { beginTransaction } from "@example/db";

export async function sameStatement0(query: TransactionQuery, id: string) {
  await query("WITH pending AS (SELECT id FROM child WHERE id = $1 FOR UPDATE) DELETE FROM child USING pending WHERE child.id = pending.id", [id]);
}
export async function sameStatement1(query: TransactionQuery, id: string) {
  await query("WITH pending AS (SELECT id FROM child WHERE id = $1 FOR UPDATE) DELETE FROM child USING pending WHERE child.id = pending.id", [id]);
}
export async function sameStatement2(query: TransactionQuery, id: string) {
  await query("WITH pending AS (SELECT id FROM child WHERE id = $1 FOR UPDATE) DELETE FROM child USING pending WHERE child.id = pending.id", [id]);
}
export async function sameStatement3(query: TransactionQuery, id: string) {
  await query("WITH pending AS (SELECT id FROM child WHERE id = $1 FOR UPDATE) DELETE FROM child USING pending WHERE child.id = pending.id", [id]);
}

export async function transactionMatched0(query: TransactionQuery, id: string) {
  await query("SELECT id FROM child WHERE id = $1 FOR UPDATE", [id]);
  await query("UPDATE child SET touched = true WHERE id = $1", [id]);
}
export async function transactionMatched1(query: TransactionQuery, id: string) {
  await query("SELECT id FROM child WHERE id = $1 FOR UPDATE", [id]);
  await query("UPDATE child SET touched = true WHERE id = $1", [id]);
}
export async function transactionMatched2(query: TransactionQuery, id: string) {
  await query("SELECT id FROM child WHERE id = $1 FOR UPDATE", [id]);
  await query("UPDATE child SET touched = true WHERE id = $1", [id]);
}
export async function transactionMatched3(query: TransactionQuery, id: string) {
  await query("SELECT id FROM child WHERE id = $1 FOR UPDATE", [id]);
  await query("UPDATE child SET touched = true WHERE id = $1", [id]);
}

export async function transactionUnmatched0(query: TransactionQuery, id: string) {
  await query("SELECT id FROM child WHERE id = $1 FOR UPDATE", [id]);
}
export async function transactionUnmatched1(query: TransactionQuery, id: string) {
  await query("SELECT id FROM child WHERE id = $1 FOR UPDATE", [id]);
}
export async function transactionUnmatched2(query: TransactionQuery, id: string) {
  await query("SELECT id FROM child WHERE id = $1 FOR UPDATE", [id]);
}
export async function transactionUnmatched3(query: TransactionQuery, id: string) {
  await query("SELECT id FROM child WHERE id = $1 FOR UPDATE", [id]);
}

export async function unionExecutor0(query: TransactionQuery | PoolQuery, id: string) {
  await query("SELECT id FROM child WHERE id = $1 FOR UPDATE", [id]);
  await query("UPDATE child SET touched = true WHERE id = $1", [id]);
}
export async function unionExecutor1(query: TransactionQuery | PoolQuery, id: string) {
  await query("SELECT id FROM child WHERE id = $1 FOR UPDATE", [id]);
  await query("UPDATE child SET touched = true WHERE id = $1", [id]);
}
export async function unionExecutor2(query: TransactionQuery | PoolQuery, id: string) {
  await query("SELECT id FROM child WHERE id = $1 FOR UPDATE", [id]);
  await query("UPDATE child SET touched = true WHERE id = $1", [id]);
}
export async function unionExecutor3(query: TransactionQuery | PoolQuery, id: string) {
  await query("SELECT id FROM child WHERE id = $1 FOR UPDATE", [id]);
  await query("UPDATE child SET touched = true WHERE id = $1", [id]);
}

export async function configuredTransaction0(id: string) {
  const tx = await beginTransaction();
  await tx.query("SELECT id FROM child WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("UPDATE child SET touched = true WHERE id = $1", [id]);
  await tx.commit();
}
