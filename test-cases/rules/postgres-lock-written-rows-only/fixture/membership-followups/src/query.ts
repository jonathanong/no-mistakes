import { beginTransaction, type PoolQuery, type TransactionQuery } from "@example/db";

export async function cleanTransaction() {
  const tx = await beginTransaction();
  await tx.query(`WITH pending AS (SELECT id FROM child WHERE id IS NOT NULL FOR UPDATE)
    DELETE FROM child USING pending WHERE child.id = pending.id`);
}

export async function anonymousPlaceholders(query: PoolQuery, first: number, second: number) {
  await query(`WITH locked AS (SELECT id FROM items WHERE id = ? FOR UPDATE),
    changed AS (UPDATE items SET value = 1 WHERE id = ? RETURNING id)
    SELECT id FROM locked`, [first, second]);
}

export async function namedPlaceholders(query: PoolQuery, id: number) {
  await query(`WITH locked AS (SELECT id FROM items WHERE id = :id FOR UPDATE),
    changed AS (UPDATE items SET value = 1 WHERE id = :id RETURNING id)
    SELECT id FROM locked`, { id });
}

export async function numberedPlaceholder(query: PoolQuery, id: number) {
  await query(`WITH locked AS (SELECT id FROM items WHERE id = $1 FOR UPDATE),
    changed AS (UPDATE items SET value = 1 WHERE id = $1 RETURNING id)
    SELECT id FROM locked`, [id]);
}

export async function returnBeforeWrite(tx: TransactionQuery, id: number) {
  await tx.query("SELECT id FROM items WHERE id = $1 FOR UPDATE", [id]);
  return;
  await tx.query("UPDATE items SET value = 1 WHERE id = $1", [id]);
}

export async function throwBeforeWrite(tx: TransactionQuery, id: number) {
  await tx.query("SELECT id FROM items WHERE id = $1 FOR UPDATE", [id]);
  throw new Error("write skipped");
  await tx.query("UPDATE items SET value = 1 WHERE id = $1", [id]);
}

export async function writeBeforeReturn(tx: TransactionQuery, id: number) {
  await tx.query("SELECT id FROM items WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("UPDATE items SET value = 1 WHERE id = $1", [id]);
  return;
}
