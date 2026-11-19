import { beginTransaction, type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Widget = { id: string; ownerId: string; state: string };
type Input = { id: string; ownerId: string; at: string };

declare function touch(value: unknown): void;

async function markWithInput(executor: SessionExecutor, input: Input) {
  await executor(
    sql`UPDATE widgets SET at = ${input.at} WHERE id = ${input.id} AND owner_id = ${input.ownerId}`,
  );
}

export async function spreadRow(tx: SessionExecutor, at: string) {
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWithInput(tx, { ...row, at });
}

export async function wholeRow(tx: SessionExecutor, at: string) {
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWithInput(tx, { id: row as never, ownerId: row.ownerId, at });
}

export async function computedKey(tx: SessionExecutor, at: string) {
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  const key = "id";
  return markWithInput(tx, { [key]: row.id, ownerId: row.ownerId, at } as Input);
}

export async function callOnRow(tx: SessionExecutor, at: string) {
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWithInput(tx, { id: row.id.toString(), ownerId: row.ownerId, at });
}

export async function writeToRow(tx: SessionExecutor, at: string) {
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWithInput(tx, { id: (row.id = "other"), ownerId: row.ownerId, at });
}

export async function mutatedLiteral(tx: SessionExecutor, at: string) {
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  const input = { id: row.id, ownerId: row.ownerId, at };
  touch(input);
  return markWithInput(tx, input);
}

export async function aliasedRow(tx: SessionExecutor, at: string) {
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  const copy = row;
  const id = copy.id;
  return markWithInput(tx, { id, ownerId: row.ownerId, at });
}

export async function mutatedFactoryLiteral(at: string) {
  const tx = await beginTransaction();
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  const input = { id: row.id, ownerId: row.ownerId, at };
  input.id = "other";
  return markWithInput(tx, input);
}
