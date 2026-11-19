import type { SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Gadget = { id: string; at: Date; tags: string[]; state: string };
type Input = { id: string; at: Date; range: { from: number; to: number } };

async function markGadget(executor: SessionExecutor, input: Input) {
  await executor(
    sql`UPDATE gadgets SET at = ${input.at}, span = ${input.range} WHERE id = ${input.id}`,
  );
}

async function markPositional(id: string, at: Date, executor: SessionExecutor) {
  await executor(sql`UPDATE gadgets SET at = ${at} WHERE id = ${id}`);
}

async function markRange(id: string, range: object, executor: SessionExecutor) {
  await executor(sql`UPDATE gadgets SET span = ${range} WHERE id = ${id}`);
}

export async function rowDateField(tx: SessionExecutor) {
  const { rows } = await tx<Gadget>(sql`SELECT id, at FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  return markGadget(tx, { id: row.id, at: row.at, range: { from: 1, to: 9 } });
}

export async function rowArrayField(tx: SessionExecutor) {
  const { rows } = await tx<Gadget>(sql`SELECT id, tags FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  return markRange(row.id, row.tags, tx);
}

export async function rowAliasProperty(tx: SessionExecutor, at: Date) {
  const { rows } = await tx<Gadget>(sql`SELECT id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  const copy = row;
  return markGadget(tx, { id: row.id, at, range: copy as never });
}

export async function rowDerivedObject(tx: SessionExecutor, at: Date) {
  const { rows } = await tx<Gadget>(sql`SELECT id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  return markGadget(tx, { id: row.id, at, range: Object.assign({}, row) as never });
}

export async function nonRowLocalObject(tx: SessionExecutor, at: Date) {
  const { rows } = await tx<Gadget>(sql`SELECT id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  const range = { from: 1, to: 2 };
  return markGadget(tx, { id: row.id, at, range });
}

export async function nonRowLocalObjectPositional(tx: SessionExecutor) {
  const { rows } = await tx<Gadget>(sql`SELECT id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  const range = { from: 1, to: 2 };
  return markRange(row.id, range, tx);
}

export async function dateMutatedWithRow(tx: SessionExecutor) {
  const { rows } = await tx<Gadget>(sql`SELECT id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  const at = new Date();
  at.setTime(row.id.length);
  return markPositional(row.id, at, tx);
}

export async function dateFromRow(tx: SessionExecutor) {
  const { rows } = await tx<Gadget>(sql`SELECT id, at FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  return markPositional(row.id, new Date(row.at), tx);
}

export async function dateDestructured(tx: SessionExecutor) {
  const { rows } = await tx<Gadget>(sql`SELECT id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  const { constructor: at } = new Date();
  return markPositional(row.id, at as never, tx);
}

export async function dateUninitialized(tx: SessionExecutor) {
  const { rows } = await tx<Gadget>(sql`SELECT id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  let at: Date | undefined;
  return markPositional(row.id, at as never, tx);
}

export async function unknownCallValue(tx: SessionExecutor, make: () => Date) {
  const { rows } = await tx<Gadget>(sql`SELECT id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  return markPositional(row.id, make(), tx);
}

export async function shadowedDate(tx: SessionExecutor, Date: new () => never) {
  const { rows } = await tx<Gadget>(sql`SELECT id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  return markPositional(row.id, new Date(), tx);
}

export async function otherConstructor(tx: SessionExecutor) {
  const { rows } = await tx<Gadget>(sql`SELECT id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  return markPositional(row.id, new Map() as never, tx);
}

export async function memberConstructor(tx: SessionExecutor) {
  const { rows } = await tx<Gadget>(sql`SELECT id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  return markPositional(row.id, new globalThis.Date(), tx);
}
