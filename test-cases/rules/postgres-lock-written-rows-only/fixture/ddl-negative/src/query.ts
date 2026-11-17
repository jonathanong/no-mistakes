import { beginTransaction } from "@example/db";
export async function noDdl() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(`SELECT 1 FROM jobs`);
}
export async function dmlOnly() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(`DELETE FROM jobs WHERE id = 1`);
}
export async function unrelated() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(`DROP TABLE events_old`);
}
export async function readBeforeLock() {
  const tx = await beginTransaction();
  await tx.query(`SELECT 1 FROM jobs; LOCK TABLE jobs IN SHARE MODE; DROP TABLE events_old`);
}
export async function readAfterDdl() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE; DROP TABLE events_old; SELECT 1 FROM jobs`);
}
export async function ddlBeforeLock() {
  const tx = await beginTransaction();
  await tx.query(`ALTER TABLE jobs ADD COLUMN title text; LOCK TABLE jobs IN SHARE MODE`);
}
export async function anotherTransaction() {
  const tx = await beginTransaction();
  const other = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await other.query(`ALTER TABLE jobs ADD COLUMN title text`);
}
export async function cteShadow() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(`WITH jobs AS (SELECT 1) SELECT * FROM jobs`);
  await tx.query(`DROP TABLE events_old`);
}
export async function wrongSchema() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE alpha.jobs IN SHARE MODE`);
  await tx.query(`ALTER TABLE beta.jobs ADD COLUMN title text`);
}
export async function dmlBeforeUnrelatedDdl() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(`UPDATE jobs SET pending = false WHERE id = 1`);
  await tx.query(`DROP TABLE events_old`);
}
export async function unawaitedRead() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  void tx.query(`SELECT 1 FROM jobs`);
  await tx.query(`DROP TABLE events_old`);
}
export async function unawaitedDdl() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  void tx.query(`ALTER TABLE jobs ADD COLUMN title text`);
}
export async function unawaitedLock() {
  const tx = await beginTransaction();
  void tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(`ALTER TABLE jobs ADD COLUMN title text`);
}
export async function cteMutation() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(`WITH changed AS (UPDATE jobs SET pending = false RETURNING id) SELECT * FROM changed`);
  await tx.query(`DROP TABLE events_old`);
}
export async function unusedCte() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(`WITH probe AS (SELECT 1 FROM jobs) SELECT 1`);
  await tx.query(`DROP TABLE events_old`);
}
export async function concurrentBodies() {
  const tx = await beginTransaction();
  await Promise.all([
    (async () => { await tx.query(`LOCK TABLE jobs IN SHARE MODE`); })(),
    (async () => { await tx.query(`ALTER TABLE jobs ADD COLUMN title text`); })(),
  ]);
}
