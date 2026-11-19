import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Widget = { id: string; ownerId: string; parts: string; nested: { id: string } };
type Input = { ownerId: string; parts: string };

async function pair(query: SessionExecutor, id: string, ownerId: string) {
  await query(sql`UPDATE widgets SET owner_id = ${ownerId} WHERE id = ${id}`);
}

async function retain(query: SessionExecutor, id: string, ownerId: string) {
  await query(sql`UPDATE widgets SET owner_id = ${ownerId} WHERE id = ${id}`);
}

async function storeParts(id: string, parts: string, query: SessionExecutor) {
  await query(sql`UPDATE widgets SET parts = ${parts} WHERE id = ${id}`);
}

async function lockWidget(query: SessionExecutor, id: string) {
  await query(sql`SELECT id FROM widgets WHERE id = ${id} FOR UPDATE`);
}

export async function q3(query: SessionExecutor, rows: Widget[], ids: string[]) {
  for (const next of ids) {
    const row = rows[0]!;
    await lockWidget(query, row.id);
    void next;
  }
}

export async function x1(query: SessionExecutor, row: Widget, input: Input) {
  await retain(query, row.id, input.ownerId);
  await storeParts(row.id, input.parts, query);
}

export async function x2(query: SessionExecutor, row: Widget, input: Input) {
  await retain(query, row.id, input.ownerId);
}

export function m1(query: SessionExecutor, widget: Widget) {
  return pair(query, widget.id, widget.ownerId);
}
