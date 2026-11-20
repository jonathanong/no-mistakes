import { beginTransaction, type SessionExecutor } from "@example/db";
import { beginTransaction as openOther } from "@example/other";
import sql from "@example/sql";

type Notice = { id: string; status: string; nested: { id: string } };
type Input = { parts: string[] };

declare function stash(value: { id: string }): void;

async function lockNotice(query: SessionExecutor, id: string) {
  await query(sql`SELECT id FROM notices WHERE id = ${id} FOR UPDATE`);
}

async function lockAll(query: SessionExecutor, ids: string[]) {
  await query(sql`UPDATE notices SET status = 'x' WHERE id = ANY(${ids})`);
}

async function dynamicPair(run: SessionExecutor, id: string, column: string) {
  await run(sql`SELECT id FROM notices WHERE id = ${id}`);
  return run(`UPDATE notices SET ${column} = 'x' WHERE id = ${id}`);
}

async function storeParts(id: string, parts: string, query: SessionExecutor) {
  await query(sql`UPDATE notices SET parts = ${parts} WHERE id = ${id}`);
}

export async function aliasInLoop(query: SessionExecutor, cache: Notice[], ids: string[]) {
  for (const id of ids) {
    const rows = cache;
    const row = rows[0]!;
    rows.push({ id, status: "x", nested: { id } });
    await lockNotice(query, row.id);
  }
}

export async function arrayParts(query: SessionExecutor, row: Notice, input: Input) {
  await storeParts(row.id, input.parts, query);
}

export async function anyRow(query: SessionExecutor, row: any) {
  return lockNotice(query, row.id);
}

export async function closureProjection(query: SessionExecutor, rows: Notice[], next: string) {
  const notice = rows[0]!;
  const run = () => lockNotice(query, notice.id);
  notice.id = next;
  return run();
}

export async function loopHeader(query: SessionExecutor, rows: Notice[]) {
  const row = rows[0]!;
  for (let index = 0; index < rows.length; lockNotice(query, row.id)) {
    row.id = "changed";
    index += 1;
  }
}

export async function nestedMember(query: SessionExecutor, row: Notice) {
  stash(row.nested);
  return lockNotice(query, row.id);
}

export async function otherExecutor(tx: SessionExecutor, other: SessionExecutor) {
  const { rows } = await other<{ rows: Notice[] }>(sql`SELECT id FROM notices`);
  await lockAll(
    tx,
    rows.map((row) => row.id),
  );
}

export async function oneDynamic(query: SessionExecutor, id: string, column: string) {
  await dynamicPair(query, id, column);
}

export async function fieldWrite(query: SessionExecutor, rows: Notice[]) {
  const notice = rows[0]!;
  class Repo {
    saved = lockNotice(query, notice.id);
  }
  notice.id = "changed";
  return new Repo();
}

export async function wrongFactory(query: SessionExecutor, id: string) {
  const tx = await openOther();
  const { rows } = (await tx(sql`SELECT id FROM notices WHERE id = ${id}`)) as {
    rows: Notice[];
  };
  return lockNotice(query, rows[0]!.id);
}

export async function shadowedFactory(query: SessionExecutor, id: string) {
  const beginTransaction = async () => query;
  const tx = await beginTransaction();
  const { rows } = await tx<{ rows: Notice[] }>(sql`SELECT id FROM notices WHERE id = ${id}`);
  return lockNotice(query, rows[0]!.id);
}

export async function capturedFactory(query: SessionExecutor, id: string) {
  const tx = await beginTransaction();
  const later = () => tx(sql`SELECT 1`);
  void later;
  const { rows } = await tx<{ rows: Notice[] }>(sql`SELECT id FROM notices WHERE id = ${id}`);
  return lockNotice(query, rows[0]!.id);
}
