import { beginTransaction } from "@example/db";
export async function conditionalDdl(enabled: boolean) {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  if (enabled) await tx.query(`ALTER TABLE jobs ADD COLUMN title text`);
}
export async function conditionalRead(enabled: boolean) {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  if (enabled) await tx.query(`SELECT 1 FROM jobs`);
  await tx.query(`DROP TABLE events_old`);
}
export async function opaqueSql(sql: string) {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(sql);
  await tx.query(`ALTER TABLE jobs ADD COLUMN title text`);
}
export async function searchPath() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(`SET search_path TO other`);
  await tx.query(`ALTER TABLE jobs ADD COLUMN title text`);
}
export async function committed() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE; COMMIT; ALTER TABLE jobs ADD COLUMN title text`);
}
export async function malformedAttach() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE events IN SHARE MODE`);
  await tx.query(`ALTER TABLE events ATTACH PARTITION`);
}
export async function malformedDetach() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE events IN SHARE MODE`);
  await tx.query(`ALTER TABLE events DETACH PARTITION events_old trailing`);
}
export async function temporaryShadow() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE events IN SHARE MODE`);
  await tx.query(`CREATE TEMP TABLE events (id integer)`);
  await tx.query(`ALTER TABLE events ADD COLUMN title text`);
}
export async function invalidPartitionKeyword() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(`ALTER TABLE jobs DETACH PART jobs_old`);
}
export async function invalidPartitionExpression() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(`ALTER TABLE jobs DETACH PARTITION 42`);
}
export async function dynamicAlterTarget(relation: string) {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(`ALTER TABLE ${relation} ADD COLUMN title text`);
}
export async function dynamicCreateTarget(relation: string) {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(`CREATE TABLE ${relation} (id integer)`);
  await tx.query(`DROP TABLE events_old`);
}
export async function dynamicPartitionTarget(relation: string) {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(`ALTER TABLE events ATTACH PARTITION ${relation} FOR VALUES FROM (1) TO (2)`);
}
