import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const sharedId = "shared";

export async function putEvent(tx: SessionExecutor, values: { id: string; at: Date }) {
  await tx(sql`INSERT INTO events (id, at) VALUES (${values.id}, ${values.at})`);
}

export async function markEach(tx: SessionExecutor, ids: string[]) {
  for (const id of ids) {
    await tx(sql`UPDATE items SET marked = true WHERE id = ${id}`);
  }
}
