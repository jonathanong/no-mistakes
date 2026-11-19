import { beginTransaction, type SessionExecutor } from "@example/db";
import sql from "@example/sql";

type Widget = { id: string; ownerId: string; state: string };
type Input = { id: string; ownerId: string; at: string };

declare function openOther(): Promise<SessionExecutor>;

async function markWithInput(executor: SessionExecutor, input: Input) {
  await executor(sql`UPDATE widgets SET at = ${input.at} WHERE id = ${input.id}`);
}

export async function unconfiguredFactory(at: string) {
  const tx = await openOther();
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWithInput(tx, { id: row.id, ownerId: row.ownerId, at });
}

export async function pendingBinding(at: string) {
  const pending = beginTransaction();
  const tx = await pending;
  const { rows } = await tx<Widget>(sql`SELECT id, owner_id FROM widgets`);
  const row = rows[0];
  if (!row) return null;
  return markWithInput(tx, { id: row.id, ownerId: row.ownerId, at });
}
