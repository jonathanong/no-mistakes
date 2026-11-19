import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Widget = { id: string; state: string };
type Result = { rows: Widget[] };

function reject(id: string, query: SessionExecutor) {
  return query(sql`SELECT ${id}`);
}

async function markWidget(id: string, query: SessionExecutor) {
  await query(sql`UPDATE widgets SET state = 'x' WHERE id = ${id}`);
}

export async function returnGuard(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  if (!row) return null;
  return markWidget(row.id, query);
}

export async function throwGuard(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  if (!row) throw new Error("missing");
  return markWidget(row.id, query);
}

export async function blockGuard(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  if (!row) {
    return null;
  }
  return markWidget(row.id, query);
}

export async function callGuard(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  if (!row) return reject(id, query);
  await markWidget(row.id, query);
  return markWidget(row.id, query);
}
