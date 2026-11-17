import { beginTransaction } from "@example/db";
export async function partitionGuard() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE public.events IN SHARE MODE`);
  await tx.query(`DROP TABLE public.events_old`);
}
export async function foreignKeyGuard() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE public.reservations IN SHARE MODE`);
  await tx.query(`DROP TABLE public.events`);
}
export async function reverseForeignKeyGuard() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE public.events IN SHARE MODE`);
  await tx.query(`ALTER TABLE public.reservations ADD COLUMN title text`);
}
export async function unrelated() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE other.events IN SHARE MODE`);
  await tx.query(`DROP TABLE public.events_old`);
}
export async function inheritedGuard() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE public.events IN SHARE MODE`);
  await tx.query(`DROP TABLE public.inherited_events`);
}
export async function externalInheritedGuard() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE public.external_guard IN SHARE MODE`);
  await tx.query(`DROP TABLE shared.parent`);
}
export async function externalForeignKeyGuard() {
  const tx = await beginTransaction();
  await tx.query(`LOCK TABLE public.external_guard IN SHARE MODE`);
  await tx.query(`DROP TABLE shared.events`);
}
