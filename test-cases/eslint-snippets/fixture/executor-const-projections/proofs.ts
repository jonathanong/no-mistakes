import assert from "fake-assert";
import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Notice = { id: string; status: string };

async function lockNotice(query: SessionExecutor, id: string) {
  await query(sql`SELECT id FROM notices WHERE id = ${id} FOR UPDATE`);
}

export function p1(query: SessionExecutor, notice: Notice) {
  return lockNotice(query, notice.id);
}

export async function k6(query: SessionExecutor, rows: Notice[]) {
  const notice = rows[0]!;
  return lockNotice(query, notice.id);
}

export async function k5(query: SessionExecutor, rows: Notice[]) {
  const notice = rows[0];
  assert(notice, 404, "not found");
  return lockNotice(query, notice.id);
}

export async function k7(query: SessionExecutor, input: Notice | undefined) {
  const notice = input;
  assert(notice, 404, "not found");
  return lockNotice(query, notice.id);
}

export async function executorRow(query: SessionExecutor, id: string) {
  const { rows } = await query(sql`UPDATE notices SET status = 'x' WHERE id = ${id} RETURNING id`);
  const updated = (rows as Notice[])[0];
  assert(updated, 404, "not found");
  return lockNotice(query, updated.id);
}
