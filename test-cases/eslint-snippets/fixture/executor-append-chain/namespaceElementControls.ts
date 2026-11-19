import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const NS_ELEMENT_COLS = sql`id, state`;

async function nsElementLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(NS_ELEMENT_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaNsElementLocal(query: SessionExecutor, id: string) {
  return nsElementLocal(query, id);
}

export { CALLED as NS_REEXPORTED_COLS } from "./fragments";
