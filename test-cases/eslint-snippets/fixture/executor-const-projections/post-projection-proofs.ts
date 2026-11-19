import assert from "fake-assert";
import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Widget = { id: string; state: string };

async function lockWidget(query: SessionExecutor, id: string) {
  await query(sql`SELECT id FROM widgets WHERE id = ${id} FOR UPDATE`);
}

async function markWidget(id: string, query: SessionExecutor) {
  await query(sql`UPDATE widgets SET state = 'x' WHERE id = ${id}`);
}

export async function h1(query: SessionExecutor, id: string) {
  const { rows } = await query(sql`UPDATE widgets SET state = 'x' WHERE id = ${id} RETURNING id`);
  const row = (rows as Widget[])[0];
  assert(row, 404, "missing");
  await lockWidget(query, row.id);
  return row;
}

export async function h2(query: SessionExecutor, id: string) {
  const { rows } = await query(sql`UPDATE widgets SET state = 'x' WHERE id = ${id} RETURNING id`);
  const row = (rows as Widget[])[0];
  assert(row, 404, "missing");
  await lockWidget(query, row.id);
  await markWidget(row.id, query);
}

export async function h3(query: SessionExecutor, id: string) {
  const { rows } = await query(sql`UPDATE widgets SET state = 'x' WHERE id = ${id} RETURNING id`);
  const row = (rows as Widget[])[0];
  assert(row, 404, "missing");
  await lockWidget(query, row.id);
  await query(sql`SELECT 1 FROM widgets WHERE id = ${row.id}`);
}

export async function h4(query: SessionExecutor, id: string) {
  const { rows } = await query(sql`UPDATE widgets SET state = 'x' WHERE id = ${id} RETURNING id`);
  const row = (rows as Widget[])[0];
  assert(row, 404, "missing");
  await lockWidget(query, row.id);
}

export async function shadowedQuery(query: SessionExecutor, rows: Widget[]) {
  const row = rows[0]!;
  {
    const query = (value: string) => value;
    await lockWidget(query as unknown as SessionExecutor, row.id);
  }
  return query;
}

export async function nonExecutorQuery(rows: Widget[], query: (text: string) => void) {
  const row = rows[0]!;
  await lockWidget(query as unknown as SessionExecutor, row.id);
}
