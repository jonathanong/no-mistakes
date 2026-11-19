import { beginTransaction, type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Widget = { id: string; state: string };

async function markWidget(id: string, executor: SessionExecutor) {
  await executor(sql`UPDATE widgets SET state = 'x' WHERE id = ${id}`);
}

export async function constRoot(id: string) {
  const tx = await beginTransaction();
  const { rows } = await tx<Widget>(sql`SELECT id FROM widgets WHERE id = ${id}`);
  const row = rows[0];
  if (!row) return null;
  return markWidget(row.id, tx);
}

export async function usingRoot(id: string) {
  await using tx = await beginTransaction();
  const { rows } = await tx<Widget>(sql`SELECT id FROM widgets WHERE id = ${id}`);
  const row = rows[0];
  if (!row) return null;
  return markWidget(row.id, tx);
}
