import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Widget = { id: string; state: string };
type Result = { rows: Widget[] };

declare function log(): void;

async function markWidget(id: string, query: SessionExecutor) {
  await query(sql`UPDATE widgets SET state = 'x' WHERE id = ${id}`);
}

export async function noExit(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  if (!row) log();
  return markWidget(row.id, query);
}

export async function otherVariable(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  const other = rows[1];
  if (!other) return null;
  return markWidget(row.id, query);
}

export async function guardAfter(query: SessionExecutor, id: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  const pending = markWidget(row.id, query);
  if (!row) return null;
  return pending;
}

export async function writeBefore(query: SessionExecutor, id: string, next: string) {
  const { rows } = (await query(sql`SELECT id FROM widgets WHERE id = ${id}`)) as Result;
  const row = rows[0];
  if (!row) return null;
  row.id = next;
  return markWidget(row.id, query);
}
