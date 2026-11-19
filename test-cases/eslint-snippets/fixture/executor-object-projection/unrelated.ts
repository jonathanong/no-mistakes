import { beginTransaction, type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Gadget = { id: string; ownerId: string | null; state: string };
type Input = { id: string; ownerId: string | null; at: Date };

async function markGadget(executor: SessionExecutor, input: Input) {
  await executor(
    sql`UPDATE gadgets SET at = ${input.at}, owner_id = ${input.ownerId} WHERE id = ${input.id}`,
  );
}

async function markPositional(id: string, at: Date, executor: SessionExecutor) {
  await executor(sql`UPDATE gadgets SET at = ${at} WHERE id = ${id}`);
}

export async function parameterDate(tx: SessionExecutor, at: Date) {
  const { rows } = await tx<Gadget>(sql`SELECT id, owner_id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  return markGadget(tx, { id: row.id, ownerId: row.ownerId, at });
}

export async function defaultedDate(tx: SessionExecutor, at = new Date()) {
  const { rows } = await tx<Gadget>(sql`SELECT id, owner_id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  return markGadget(tx, { id: row.id, ownerId: row.ownerId, at });
}

export async function inlineDate(tx: SessionExecutor) {
  const { rows } = await tx<Gadget>(sql`SELECT id, owner_id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  return markGadget(tx, { id: row.id, ownerId: row.ownerId, at: new Date() });
}

export async function localDate(tx: SessionExecutor) {
  const { rows } = await tx<Gadget>(sql`SELECT id, owner_id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  const at = new Date(2020, 1, 1);
  return markGadget(tx, { id: row.id, ownerId: row.ownerId, at });
}

export async function factoryDate(reversedAt = new Date()) {
  const tx = await beginTransaction();
  const { rows } = await tx<Gadget>(sql`SELECT id, owner_id FROM gadgets`);
  const found = rows[0];
  if (!found) return null;
  return markGadget(tx, { id: found.id, ownerId: found.ownerId, at: reversedAt });
}

export async function positionalParameter(tx: SessionExecutor, at: Date) {
  const { rows } = await tx<Gadget>(sql`SELECT id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  return markPositional(row.id, at, tx);
}

export async function positionalFresh(tx: SessionExecutor) {
  const { rows } = await tx<Gadget>(sql`SELECT id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  return markPositional(row.id, new Date(), tx);
}

export async function positionalNullable(tx: SessionExecutor, at: Date | null) {
  const { rows } = await tx<Gadget>(sql`SELECT id FROM gadgets`);
  const row = rows[0];
  if (!row) return null;
  return markPositional(row.id, at as Date, tx);
}
