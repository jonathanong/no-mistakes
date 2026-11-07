import { beginTransaction } from "@example/db";
export async function matchedAlias(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT i.id FROM items AS i WHERE i.id = $1 FOR UPDATE OF i`, [id]);
  await tx.query(`UPDATE items AS target SET value = 1 WHERE target.id = $1`, [id]);
}
export async function mismatchedAlias(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT i.id FROM items AS i WHERE i.id = $1 FOR UPDATE OF wrong`, [id]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
export async function derivedRelation() {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM (SELECT id FROM items) AS derived FOR UPDATE`);
}
export async function joinedRelations() {
  const tx = await beginTransaction();
  await tx.query(`SELECT i.id FROM items AS i JOIN parents AS p ON p.id = i.parent_id FOR UPDATE OF i`);
}
export async function advisoryForms(id: number) {
  const tx = await beginTransaction();
  await tx.query(`SELECT pg_catalog.pg_advisory_lock_shared(42::bigint)`);
  await tx.query(`SELECT pg_advisory_xact_lock_shared(-42)`);
  await tx.query(`SELECT pg_try_advisory_lock_shared(40 + 2)`);
  await tx.query(`SELECT pg_try_advisory_xact_lock(1, 2)`);
  await tx.query(`SELECT pg_advisory_lock_shared(1, 2)`);
  await tx.query(`SELECT pg_advisory_xact_lock_shared(1, 2)`);
  await tx.query(`SELECT pg_advisory_xact_lock(hashtext($1))`, [id]);
  await tx.query(`UPDATE items SET value = 1 WHERE id = $1`, [id]);
}
export async function ambiguousQualification(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(`UPDATE app.items SET value = 1 WHERE id = $1`, [id]);
}
export async function exactQualification(id: string) {
  const tx = await beginTransaction();
  await tx.query(`SELECT id FROM app.items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(`UPDATE app.items SET value = 1 WHERE id = $1`, [id]);
}
