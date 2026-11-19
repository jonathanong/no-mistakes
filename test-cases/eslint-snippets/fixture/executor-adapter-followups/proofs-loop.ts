import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Notice = { id: string; status: string };

async function lockAll(query: SessionExecutor, ids: string[]) {
  await query(sql`UPDATE notices SET status = 'x' WHERE id = ANY(${ids})`);
}

async function lockNotice(query: SessionExecutor, id: string) {
  await query(sql`SELECT id FROM notices WHERE id = ${id} FOR UPDATE`);
}

async function twoCalls(run: SessionExecutor, id: string) {
  await run(sql`SELECT id FROM notices WHERE id = ${id}`);
  return run(sql`UPDATE notices SET status = 'x' WHERE id = ${id}`);
}

export async function freshQuery(query: SessionExecutor, ids: string[]) {
  for (const id of ids) {
    const { rows } = await query<{ rows: Notice[] }>(sql`SELECT id FROM notices WHERE id = ${id}`);
    const row = rows[0]!;
    await lockNotice(query, row.id);
  }
}

export async function sameExecutor(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Notice[] }>(sql`SELECT id FROM notices`);
  await lockAll(
    tx,
    rows.map((row) => row.id),
  );
}

export async function mutateAfter(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Notice[] }>(sql`SELECT id FROM notices`);
  const ids = rows.map((row) => row.id);
  await lockAll(tx, ids);
  ids.push("later");
}

export async function twoStatements(query: SessionExecutor, id: string) {
  await twoCalls(query, id);
}

export async function primitiveLoop(query: SessionExecutor, ids: string[]) {
  for (const id of ids) await lockNotice(query, id);
}

export function fieldLater(query: SessionExecutor, rows: Notice[]) {
  const notice = rows[0]!;
  return class {
    saved = lockNotice(query, notice.id);
  };
}
