import { beginTransaction } from "@example/db";
export async function detachPreflight() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE events IN ACCESS EXCLUSIVE MODE`);
  await tx.query(`SELECT 1 FROM reservations LIMIT 1`);
  await tx.query(`ALTER TABLE events DETACH PARTITION events_old`);
}
export async function defaultMoveAttach() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE events IN ACCESS EXCLUSIVE MODE`);
  await tx.query(`WITH moved AS (DELETE FROM events_default WHERE id = 1 RETURNING *) INSERT INTO events_new SELECT * FROM moved`);
  await tx.query(`ALTER TABLE events ATTACH PARTITION events_new FOR VALUES FROM (1) TO (2)`);
}
export async function relatedGuardDrop() {
  const tx = await beginTransaction();
  await tx.query(`CREATE TABLE guards (id integer REFERENCES events(id))`);
  await tx.query(`LOCK TABLE guards IN SHARE MODE`);
  await tx.query(`DROP TABLE events`);
}
export async function inheritedGuardDrop() {
  const tx = await beginTransaction();
  await tx.query(`CREATE TABLE child () INHERITS (parent)`);
  await tx.query(`LOCK TABLE parent IN SHARE MODE`);
  await tx.query(`DROP TABLE child`);
}
export async function partitionDrop() {
  const tx = await beginTransaction();
  await tx.query(`CREATE TABLE child PARTITION OF parent FOR VALUES FROM (1) TO (2)`);
  await tx.query(`LOCK TABLE parent IN SHARE MODE`);
  await tx.query(`DROP TABLE child`);
}
export async function sameCall() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE events IN SHARE MODE; ALTER TABLE events ADD COLUMN title text`);
}
export async function batchedDetach() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE events IN SHARE MODE; SELECT 1 FROM reservations; ALTER TABLE events DETACH PARTITION events_old`);
}
export async function tableConstraintGuardDrop() {
  const tx = await beginTransaction();
  await tx.query(`CREATE TABLE guards_table (id integer NOT NULL, event_id integer, PRIMARY KEY (id), FOREIGN KEY (event_id) REFERENCES events(id))`);
  await tx.query(`LOCK TABLE guards_table IN SHARE MODE`);
  await tx.query(`DROP TABLE events`);
}
export async function qualifiedPartitionDetach() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE public.events_old IN SHARE MODE`);
  await tx.query(`ALTER TABLE public.events DETACH PARTITION public.events_old`);
}
