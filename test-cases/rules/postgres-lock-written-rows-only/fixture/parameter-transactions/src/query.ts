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
    SELECT id FROM child WHERE id IS NOT NULL FOR UPDATE
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

export async function nullableJoin(query: PoolQuery) {
  await query(`WITH pending AS (SELECT owner_id FROM items WHERE owner_id IS NULL FOR UPDATE)
    DELETE FROM items USING pending WHERE items.owner_id = pending.owner_id`);
}

export async function explicitNonNullJoin(query: PoolQuery) {
  await query(`WITH pending AS (SELECT owner_id FROM items WHERE owner_id IS NOT NULL FOR UPDATE)
    DELETE FROM items USING pending WHERE items.owner_id = pending.owner_id`);
}

export async function nullSafeJoin(query: PoolQuery) {
  await query(`WITH pending AS (SELECT owner_id FROM items WHERE owner_id IS NULL FOR UPDATE)
    DELETE FROM items USING pending WHERE items.owner_id IS NOT DISTINCT FROM pending.owner_id`);
}

export async function catalogMayBeStale(query: PoolQuery) {
  await query("ALTER TABLE mutable_items ALTER COLUMN id DROP NOT NULL");
  await query(`WITH pending AS (SELECT id FROM mutable_items FOR UPDATE)
    DELETE FROM mutable_items USING pending WHERE mutable_items.id = pending.id`);
}

export async function catalogOnlyAfterOpaqueSql(opaque: string) {
  const tx = await beginTransaction();
  await tx.query(opaque);
  await tx.query(`WITH pending AS (SELECT id FROM child FOR UPDATE)
    DELETE FROM child USING pending WHERE child.id = pending.id`);
}

export async function equalityProvesNonNull(query: PoolQuery, id: number) {
  await query(`WITH pending AS (SELECT owner_id FROM items WHERE owner_id = $1 FOR UPDATE)
    DELETE FROM items USING pending WHERE items.owner_id = pending.owner_id`, [id]);
}

export async function reverseEqualityProvesNonNull(query: PoolQuery, id: number) {
  await query(`WITH pending AS (SELECT i.owner_id FROM items i WHERE $1 = i.owner_id FOR UPDATE)
    DELETE FROM items USING pending WHERE items.owner_id = pending.owner_id`, [id]);
}

export async function conjunctionProvesNonNull(query: PoolQuery) {
  await query(`WITH pending AS (SELECT owner_id FROM items WHERE value > 0 AND owner_id IS NOT NULL FOR UPDATE)
    DELETE FROM items USING pending WHERE items.owner_id = pending.owner_id`);
}

export async function coercingObjectKey(tx: Tx) {
  let next = 1;
  const key = { toPostgres: () => next++ };
  await tx.query("SELECT id FROM items WHERE id = $1 FOR UPDATE", [key]);
  await tx.query("UPDATE items SET value = 1 WHERE id = $1", [key]);
}

export async function changingObjectKey(tx: Tx, key: { value: number }) {
  await tx.query("SELECT id FROM items WHERE id = $1 FOR UPDATE", [key]);
  key.value += 1;
  await tx.query("UPDATE items SET value = 1 WHERE id = $1", [key]);
}

export async function primitiveKeyWithPayload(tx: Tx, id: number, payload: object) {
  await tx.query("SELECT id FROM items WHERE id = $1 FOR UPDATE", [id]);
  await tx.query("UPDATE items SET value = $1 WHERE id = $2", [payload, id]);
}

export async function differentCoercionSlots(query: PoolQuery) {
  let next = 1;
  const key = { toPostgres: () => next++ };
  await query(`WITH locked AS (SELECT id FROM items WHERE id = $1 FOR UPDATE),
    changed AS (UPDATE items SET value = 1 WHERE id = $2 RETURNING id)
    SELECT id FROM locked`, [key, key]);
}

export async function sameCoercionSlot(query: PoolQuery) {
  let next = 1;
  const key = { toPostgres: () => next++ };
  await query(`WITH locked AS (SELECT id FROM items WHERE id = $1 FOR UPDATE),
    changed AS (UPDATE items SET value = 1 WHERE id = $1 RETURNING id)
    SELECT id FROM locked`, [key]);
}

export async function differentArrayCasts(tx: Tx, id: number) {
  await tx.query("SELECT id FROM items WHERE id = $1::integer FOR UPDATE", [id]);
  await tx.query("UPDATE items SET value = 1 WHERE id = $1::numeric", [id]);
}

export async function differentTagCasts(tx: Tx, id: number) {
  await tx.query(sql`SELECT id FROM items WHERE id = ${id}::integer FOR UPDATE`);
  await tx.query(sql`UPDATE items SET value = 1 WHERE id = ${id}::numeric`);
}

export async function differentStatementCasts(query: PoolQuery, id: number) {
  await query(`WITH locked AS (SELECT id FROM items WHERE id = $1::integer FOR UPDATE),
    changed AS (UPDATE items SET value = 1 WHERE id = $1::numeric RETURNING id)
    SELECT id FROM locked`, [id]);
}

export async function castOnOneLockTarget(tx: Tx, item: number, child: number) {
  await tx.query(`SELECT 1 FROM items x JOIN child y ON y.parent_id = x.id
    WHERE x.id = $1::integer AND y.id = $2 FOR UPDATE OF x FOR UPDATE OF y`, [item, child]);
  await tx.query("UPDATE items SET value = 1 WHERE id = $1", [item]);
  await tx.query("UPDATE child SET value = 1 WHERE id = $1", [child]);
}

export async function plainKeysOnBothLockTargets(tx: Tx, item: number, child: number) {
  await tx.query(`SELECT 1 FROM items x JOIN child y ON y.parent_id = x.id
    WHERE x.id = $1 AND y.id = $2 FOR UPDATE OF x FOR UPDATE OF y`, [item, child]);
  await tx.query("UPDATE items SET value = 1 WHERE id = $1", [item]);
  await tx.query("UPDATE child SET value = 1 WHERE id = $1", [child]);
}
