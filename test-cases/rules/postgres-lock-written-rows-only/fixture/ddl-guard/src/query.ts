import { beginTransaction } from "@example/db";
export async function finalProof() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE reservations IN SHARE MODE`);
  await tx.query(`SELECT 1 FROM reservations WHERE id = 1 LIMIT 1`);
  await tx.query(`ALTER TABLE events DETACH PARTITION events_old`);
  await tx.query(`DROP TABLE events_old`);
}
export async function pendingProbe() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE`);
  await tx.query(`SELECT 1 FROM jobs WHERE pending LIMIT 1`);
  await tx.query(`DROP TABLE events_old`);
}
export async function sameCall() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs IN SHARE MODE; SELECT 1 FROM jobs; DROP TABLE events_old`);
}
export async function joinedProof() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE jobs, reservations IN SHARE MODE`);
  await tx.query(`SELECT 1 FROM jobs JOIN reservations ON jobs.id = reservations.id LIMIT 1`);
  await tx.query(`DROP TABLE events_old`);
}
