import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import * as helpers from "./helpers";

type Notice = { id: string; status: string };

async function lockNotice(query: SessionExecutor, id: string) {
  await query(sql`SELECT id FROM notices WHERE id = ${id} FOR UPDATE`);
}

async function lockAll(query: SessionExecutor, ids: string[]) {
  await query(sql`UPDATE notices SET status = 'x' WHERE id = ANY(${ids})`);
}

export async function letElement(query: SessionExecutor, rows: Notice[]) {
  let notice = rows[0]!;
  const first = rows[1]!;
  await lockNotice(query, notice.id);
  return lockNotice(query, first.id);
}

export async function plainRows(query: SessionExecutor) {
  const bag = { rows: [] as Notice[] };
  const { rows } = bag;
  const row = rows[0]!;
  return lockNotice(query, row.id);
}

export async function fieldLoop(query: SessionExecutor, rows: Notice[]) {
  await helpers.eachField(rows, query);
}

export async function reducedField(query: SessionExecutor, rows: Notice[]) {
  await rows.reduce(({ id }) => lockNotice(query, id), Promise.resolve());
}

export async function storedCallback(query: SessionExecutor, rows: Notice[]) {
  const mark = ({ id }) => lockNotice(query, id);
  await Promise.all(rows.map(mark));
}

export async function freshDestructure(query: SessionExecutor, ids: string[]) {
  for (const id of ids) {
    const { rows } = await query<{ rows: Notice[] }>(sql`SELECT id FROM notices WHERE id = ${id}`);
    const row = rows[0]!;
    await lockNotice(query, row.id);
    rows.push({ id, status: "x" });
  }
}

export async function awaitedIds(tx: SessionExecutor) {
  const { rows } = await tx<{ rows: Notice[] }>(sql`SELECT id FROM notices`);
  const ids = rows.map((row) => row.id);
  await lockAll(tx, await ids);
}

export async function unknownClosure(query: SessionExecutor, id: string) {
  await helpers.badClosure(id, { query });
}

export async function assignBefore(query: SessionExecutor, other: SessionExecutor, id: string) {
  query = other;
  const { rows } = await query<{ rows: Notice[] }>(sql`SELECT id FROM notices WHERE id = ${id}`);
  const row = rows[0]!;
  return lockNotice(query, row.id);
}

export async function outerRoot(query: SessionExecutor, ids: string[], outside: Notice[]) {
  for (const id of ids) {
    const rows = outside;
    const row = rows[0]!;
    await lockNotice(query, row.id);
    rows.push({ id, status: "x" });
  }
}

export async function staleAlias(query: SessionExecutor, ids: string[], outside: Notice[]) {
  for (const id of ids) {
    const fetched = outside;
    const rows = fetched;
    const row = rows[0]!;
    await lockNotice(query, row.id);
    rows.push({ id, status: "x" });
  }
}
