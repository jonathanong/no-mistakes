import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Widget = { id: string; ownerId: string; nested: { id: string } };

declare function stash(value: unknown): void;

async function pair(query: SessionExecutor, id: string, ownerId: string) {
  await query(sql`UPDATE widgets SET owner_id = ${ownerId} WHERE id = ${id}`);
}

async function lockWidget(query: SessionExecutor, id: string) {
  await query(sql`SELECT id FROM widgets WHERE id = ${id} FOR UPDATE`);
}

export async function q1(query: SessionExecutor, rows: Widget[], ids: string[]) {
  for (const next of ids) {
    const row = rows[0]!;
    await lockWidget(query, row.id);
    row.id = next;
  }
}

export async function q2(query: SessionExecutor, rows: Widget[]) {
  for (let i = 0; i < rows.length; i++) {
    const row = rows[i]!;
    await lockWidget(query, row.id);
    row.id = "x";
  }
}

export async function paramEscapeBefore(query: SessionExecutor, widget: Widget) {
  stash(widget);
  await pair(query, widget.id, widget.ownerId);
}

export async function paramLoopWrite(query: SessionExecutor, widget: Widget, ids: string[]) {
  for (const next of ids) {
    await lockWidget(query, widget.id);
    widget.id = next;
  }
}

export async function paramObjectArgBefore(query: SessionExecutor, widget: Widget) {
  stash(widget.nested);
  await lockWidget(query, widget.id);
}
