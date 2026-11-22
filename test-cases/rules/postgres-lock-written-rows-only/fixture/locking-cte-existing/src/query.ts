import { beginTransaction } from "@example/db";

export async function writesOtherRelation(id: number) {
  await using query = await beginTransaction();
  await query(`WITH parent AS MATERIALIZED (SELECT o.id FROM orders AS o WHERE o.id = $1 FOR UPDATE OF o) INSERT INTO order_events (order_id) SELECT id FROM parent`, [id]);
}

export async function laterWritesLockedRelation(id: number) {
  await using query = await beginTransaction();
  await query(`WITH parent AS MATERIALIZED (SELECT o.id FROM orders AS o WHERE o.id = $1 FOR UPDATE OF o) INSERT INTO order_events (order_id) SELECT id FROM parent`, [id]);
  await query(`UPDATE orders SET touched = true WHERE id = $1`, [id]);
}

export async function lockOnly(id: number) {
  await using query = await beginTransaction();
  await query(`SELECT o.id FROM orders AS o WHERE o.id = $1 FOR UPDATE OF o`, [id]);
}
