import assert from "fake-assert";
import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Widget = { id: string; state: string };

declare function stash(widget: Widget): void;
declare function lookupRows(): Promise<{ rows: Widget[] }>;

async function lockWidget(query: SessionExecutor, id: string) {
  await query(sql`SELECT id FROM widgets WHERE id = ${id} FOR UPDATE`);
}

async function markWidget(id: string, query: SessionExecutor) {
  await query(sql`UPDATE widgets SET state = 'x' WHERE id = ${id}`);
}

export async function writeBefore(query: SessionExecutor, rows: Widget[], next: string) {
  const row = rows[0]!;
  row.id = next;
  await lockWidget(query, row.id);
}

export async function escapeBefore(query: SessionExecutor, rows: Widget[]) {
  const row = rows[0]!;
  stash(row);
  await lockWidget(query, row.id);
}

export async function aliasBefore(query: SessionExecutor, rows: Widget[]) {
  const row = rows[0]!;
  const alias = row;
  await lockWidget(query, row.id);
  return alias;
}

export async function writeBetween(query: SessionExecutor, rows: Widget[], next: string) {
  const row = rows[0]!;
  await lockWidget(query, row.id);
  row.id = next;
  await markWidget(row.id, query);
}

export async function loopWrite(query: SessionExecutor, rows: Widget[], next: string) {
  const row = rows[0]!;
  for (let index = 0; index < 2; index++) {
    await lockWidget(query, row.id);
    row.id = next;
  }
}

export async function closureWrite(query: SessionExecutor, rows: Widget[], next: string) {
  const row = rows[0]!;
  const rename = () => {
    row.id = next;
  };
  await lockWidget(query, row.id);
  rename();
}

export async function otherSource(query: SessionExecutor) {
  const { rows } = await lookupRows();
  const row = rows[0]!;
  assert(row, 404, "missing");
  await lockWidget(query, row.id);
  return row;
}
