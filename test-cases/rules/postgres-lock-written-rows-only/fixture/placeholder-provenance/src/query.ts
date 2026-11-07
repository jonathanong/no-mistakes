import { beginTransaction } from "@example/db";
import { sql } from "@example/sql";
export async function dynamicRelations(first: string, second: string, id: string) {
  const tx = await beginTransaction();
  await tx.query(sql`SELECT id FROM ${first} WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(sql`UPDATE ${second} SET value = 1 WHERE id = $1`, [id]);
}
export async function opaqueWrite(second: string, id: string) {
  const tx = await beginTransaction();
  await tx.query(sql`SELECT id FROM items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(sql`UPDATE ${second} SET value = 1 WHERE id = $1`, [id]);
}
export async function authoredMarker(id: string) {
  const tx = await beginTransaction();
  await tx.query(sql`SELECT id FROM sql_placeholder_1 WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(sql`UPDATE sql_placeholder_1 SET value = 1 WHERE id = $1`, [id]);
}
export async function dynamicSchema(schema: string, id: string) {
  const tx = await beginTransaction();
  await tx.query(sql`SELECT id FROM ${schema}.items WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(sql`UPDATE ${schema}.items SET value = 1 WHERE id = $1`, [id]);
}
export async function quotedAuthoredMarker(id: string) {
  const tx = await beginTransaction();
  await tx.query(sql`SELECT id FROM app."sql_placeholder_1" WHERE id = $1 FOR UPDATE`, [id]);
  await tx.query(sql`UPDATE app."sql_placeholder_1" SET value = 1 WHERE id = $1`, [id]);
}
export async function dynamicKeyColumns(first: string, second: string) {
  const tx = await beginTransaction();
  await tx.query(sql`SELECT id FROM items WHERE ${first} = 1 FOR UPDATE`);
  await tx.query(sql`UPDATE items SET value = 1 WHERE ${second} = 1`);
}
export async function authoredMarkerColumn() {
  const tx = await beginTransaction();
  await tx.query(sql`SELECT id FROM items WHERE sql_placeholder_1 = 1 FOR UPDATE`);
  await tx.query(sql`UPDATE items SET value = 1 WHERE sql_placeholder_1 = 1`);
}
export async function dynamicAliases(first: string, second: string) {
  const tx = await beginTransaction();
  await tx.query(sql`SELECT id FROM items AS ${first} WHERE id = 1 FOR UPDATE`);
  await tx.query(sql`UPDATE items AS ${second} SET value = 1 WHERE id = 1`);
}
export async function authoredMarkerAlias() {
  const tx = await beginTransaction();
  await tx.query(sql`SELECT sql_placeholder_1.id FROM items AS sql_placeholder_1 WHERE sql_placeholder_1.id = 1 FOR UPDATE OF sql_placeholder_1`);
  await tx.query(sql`UPDATE items AS sql_placeholder_1 SET value = 1 WHERE sql_placeholder_1.id = 1`);
}
export async function dynamicLockTarget(target: string) {
  const tx = await beginTransaction();
  await tx.query(sql`SELECT id FROM sql_placeholder_1 WHERE id = 1 FOR UPDATE OF ${target}`);
  await tx.query(sql`UPDATE sql_placeholder_1 SET value = 1 WHERE id = 1`);
}
export async function authoredLockTarget() {
  const tx = await beginTransaction();
  await tx.query(sql`SELECT id FROM sql_placeholder_1 WHERE id = 1 FOR UPDATE OF sql_placeholder_1`);
  await tx.query(sql`UPDATE sql_placeholder_1 SET value = 1 WHERE id = 1`);
}
