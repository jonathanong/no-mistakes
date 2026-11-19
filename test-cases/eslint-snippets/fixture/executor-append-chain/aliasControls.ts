import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import { ALIASED, DEFAULTED } from "./aliasSource";

async function renamedMutation(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(ALIASED).append(sql` FROM widgets WHERE id = ${id}`));
}

async function defaultMutation(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(DEFAULTED).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaRenamed(query: SessionExecutor, id: string) {
  return renamedMutation(query, id);
}
export function viaDefault(query: SessionExecutor, id: string) {
  return defaultMutation(query, id);
}
