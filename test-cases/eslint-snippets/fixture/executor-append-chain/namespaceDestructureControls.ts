import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const NS_DESTRUCTURED_COLS = sql`id, state`;

async function nsDestructuredLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(NS_DESTRUCTURED_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaNsDestructuredLocal(query: SessionExecutor, id: string) {
  return nsDestructuredLocal(query, id);
}
