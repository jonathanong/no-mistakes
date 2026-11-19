import { beginTransaction, type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import { markImported, markPositional } from "./helper";

type Widget = { id: string; ownerId: string; state: string };
type Input = { id: string; ownerId: string; at: string };

async function markWithInput(executor: SessionExecutor, input: Input) {
  await executor(
    sql`UPDATE widgets SET at = ${input.at} WHERE id = ${input.id} AND owner_id = ${input.ownerId}`,
  );
}

export async function parameterRoot(tx: SessionExecutor, at: string) {
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWithInput(tx, { id: row.id, ownerId: row.ownerId, at });
}

export async function factoryRoot(at: string) {
  const tx = await beginTransaction();
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWithInput(tx, { id: row.id, ownerId: row.ownerId, at });
}

export async function shorthandProjection(tx: SessionExecutor, at: string) {
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  const id = row.id;
  const ownerId = row.ownerId;
  return markWithInput(tx, { id, ownerId, at });
}

export async function importedHelper(tx: SessionExecutor, at: string) {
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markImported(tx, { id: row.id, ownerId: row.ownerId, at });
}

export async function factoryShorthand(at: string) {
  const tx = await beginTransaction();
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  const id = row.id;
  const ownerId = row.ownerId;
  return markWithInput(tx, { id, ownerId, at });
}

export async function positionalAlias(tx: SessionExecutor) {
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  const id = row.id;
  return markPositional(id, tx);
}
