import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Notice = { id: string; status: string };

async function lockNotice(query: SessionExecutor, id: string) {
  await query(sql`SELECT id FROM notices WHERE id = ${id} FOR UPDATE`);
}

export async function twoElements(query: SessionExecutor, rows: Notice[]) {
  const first = rows[0]!;
  const second = rows[1]!;
  await lockNotice(query, first.id);
  return lockNotice(query, second.id);
}

export async function secondLocal(query: SessionExecutor, rows: Notice[]) {
  const first = rows[0]!;
  const notice = first;
  return lockNotice(query, notice.id);
}

export async function scalarCopy(query: SessionExecutor, rows: Notice[]) {
  const first = rows[0]!;
  const id = first.id;
  const second = rows[1]!;
  await lockNotice(query, id);
  return lockNotice(query, second.id);
}

export async function executorRows(query: SessionExecutor, id: string) {
  const { rows } = await query<{ rows: Notice[] }>(
    sql`UPDATE notices SET status = 'x' WHERE id = ${id} RETURNING id`,
  );
  const updated = rows[0]!;
  return lockNotice(query, updated.id);
}
