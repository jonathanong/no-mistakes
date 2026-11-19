import { beginTransaction, openSession, type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Widget = { id: string; state: string };

declare function keep(executor: SessionExecutor): void;

async function markWidget(id: string, executor: SessionExecutor) {
  await executor(sql`UPDATE widgets SET state = 'x' WHERE id = ${id}`);
}

export async function reassigned(id: string) {
  let tx = await beginTransaction();
  const { rows } = await tx<Widget>(sql`SELECT id FROM widgets WHERE id = ${id}`);
  const row = rows[0];
  if (!row) return null;
  tx = await beginTransaction();
  return markWidget(row.id, tx);
}

export async function unconfigured(id: string, query: SessionExecutor) {
  const tx = await openSession();
  const { rows } = await tx<Widget>(sql`SELECT id FROM widgets WHERE id = ${id}`);
  const row = rows[0];
  if (!row) return null;
  return markWidget(row.id, query);
}

export async function escapes(id: string) {
  const tx = await beginTransaction();
  keep(tx);
  const { rows } = await tx<Widget>(sql`SELECT id FROM widgets WHERE id = ${id}`);
  const row = rows[0];
  if (!row) return null;
  return markWidget(row.id, tx);
}

export async function writesRow(id: string) {
  const tx = await beginTransaction();
  const { rows } = await tx<Widget>(sql`SELECT id FROM widgets WHERE id = ${id}`);
  const row = rows[0];
  if (!row) return null;
  row.id = "other";
  return markWidget(row.id, tx);
}
