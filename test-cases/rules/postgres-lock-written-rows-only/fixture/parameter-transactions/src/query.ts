import type { PoolQuery, TransactionQuery as Tx } from "@example/db";
import { beginTransaction, sql } from "@example/db";

export async function drain(query: PoolQuery, parent: number) {
  await query(sql`WITH pending AS (
    SELECT c.id FROM child c WHERE c.parent_id = ${parent}
    ORDER BY c.id LIMIT 10 FOR UPDATE OF c
  ) DELETE FROM child c USING pending WHERE c.id = pending.id`);
}

export async function write(tx: Tx, id: number) {
  await tx.query("SELECT id FROM items WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("UPDATE items SET value = 1 WHERE id = $1", [id]);
}

export async function lockOnly(tx: Tx) {
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
}

export async function mixed(tx: PoolQuery | Tx) {
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  await tx.query("UPDATE items SET value = 1 WHERE id = 1");
}

export async function differentClient(tx: Tx, other: Tx) {
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  await other.query("UPDATE items SET value = 1 WHERE id = 1");
}

export async function earlierWrite(tx: Tx) {
  await tx.query("UPDATE items SET value = 1 WHERE id = 1");
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
}

export async function conditionalWrite(tx: Tx, writes: boolean) {
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  if (writes) {
    await tx.query("UPDATE items SET value = 1 WHERE id = 1");
  }
}

export async function closed(tx: Tx) {
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  await tx.commit();
  await tx.query("UPDATE items SET value = 1 WHERE id = 1");
}

export async function replaced(tx: Tx, other: Tx) {
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  tx = other;
  await tx.query("UPDATE items SET value = 1 WHERE id = 1");
}

export async function nested(tx: Tx) {
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  return async () => {
    await tx.query("UPDATE items SET value = 1 WHERE id = 1");
  };
}

export async function shadowed<Tx extends PoolQuery>(tx: Tx) {
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  await tx.query("UPDATE items SET value = 1 WHERE id = 1");
}

export async function wrongRow(query: PoolQuery) {
  await query(`WITH locked AS (SELECT id FROM items WHERE id = 1 FOR UPDATE),
    changed AS (UPDATE items SET value = 1 WHERE id = 2 RETURNING id)
    SELECT id FROM locked`);
}

export async function exitsAfterClosing(tx: Tx, stop: boolean) {
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  if (stop) {
    await tx.commit();
    return;
  }
  await tx.query("UPDATE items SET value = 1 WHERE id = 1");
}

export async function transactionBoundary(tx: Tx) {
  await tx.query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
  await tx.query("ROLLBACK AND CHAIN");
  await tx.query("UPDATE items SET value = 1 WHERE id = 1");
}

async function lockFromHelper(query: Tx) {
  await query("SELECT id FROM items WHERE id = 1 FOR UPDATE");
}

export async function ownedFactory() {
  const tx = await beginTransaction();
  await lockFromHelper(tx.query);
}

export async function tagged(tx: Tx, id: number) {
  await tx.query(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`);
  await tx.query(sql`UPDATE items SET value = 1 WHERE id = ${id}`);
}

export async function mixedBindingSyntax(tx: Tx, id: number) {
  await tx.query(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`);
  await tx.query("UPDATE items SET value = 1 WHERE id = $1", [id]);
}

export async function authoredPlaceholder(tx: Tx, id: number) {
  await tx.query(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`);
  await tx.query(sql`UPDATE items SET value = ${id} WHERE id = $1`);
}

export async function longerAuthoredPlaceholder(tx: Tx, id: number) {
  await tx.query(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`);
  await tx.query(sql`UPDATE items SET value = ${id} WHERE id = $10`);
}

export async function changedKey(tx: Tx, id: number) {
  await tx.query(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`);
  id += 1;
  await tx.query(sql`UPDATE items SET value = 1 WHERE id = ${id}`);
}

export async function differentLiterals(tx: Tx) {
  await tx.query(sql`SELECT id FROM items WHERE id = ${1} FOR UPDATE`);
  await tx.query(sql`UPDATE items SET value = 1 WHERE id = ${2}`);
}

export async function atomicWithOpaqueSql(opaque: string) {
  const tx = await beginTransaction();
  await tx.query(opaque);
  await tx.query(`WITH pending AS (
    SELECT id FROM child FOR UPDATE
  ) DELETE FROM child USING pending WHERE child.id = pending.id`);
}

export async function gluedPlaceholder(tx: Tx, id: number) {
  await tx.query(sql`SELECT id FROM items WHERE id = ${id} FOR UPDATE`);
  await tx.query(sql`UPDATE items SET value = 1 WHERE id = ${id}0`);
}

export async function atomicMismatchWithOpaqueSql(opaque: string) {
  const tx = await beginTransaction();
  await tx.query(opaque);
  await tx.query(`WITH locked AS (SELECT id FROM items WHERE id = 1 FOR UPDATE),
    changed AS (UPDATE items SET value = 1 WHERE id = 2 RETURNING id)
    SELECT id FROM locked`);
}

export async function differentPredicate(query: PoolQuery) {
  await query(`WITH locked AS (SELECT id FROM items WHERE id = 1 FOR UPDATE),
    changed AS (UPDATE items SET value = 1 WHERE owner_id = 1 RETURNING id)
    SELECT id FROM locked`);
}

export async function differentJoinColumn(query: PoolQuery) {
  await query(`WITH pending AS (SELECT id FROM items FOR UPDATE)
    DELETE FROM items USING pending WHERE items.owner_id = pending.id`);
}

export async function skippedLockedRows(query: PoolQuery) {
  await query(`WITH pending AS (SELECT id FROM items ORDER BY id OFFSET 1 FOR UPDATE)
    DELETE FROM items USING pending WHERE items.id = pending.id`);
}

export async function renamedKey(tx: Tx, id: number) {
  await tx.query("SELECT 1 FROM items AS i(name, id) WHERE i.id = $1 FOR UPDATE", [id]);
  await tx.query("UPDATE items SET value = 1 WHERE id = $1", [id]);
}

export async function ordinaryAlias(tx: Tx, id: number) {
  await tx.query("SELECT 1 FROM items AS i WHERE i.id = $1 FOR UPDATE", [id]);
  await tx.query("UPDATE items SET value = 1 WHERE id = $1", [id]);
}
