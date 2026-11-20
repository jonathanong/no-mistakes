import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const SIBLING_COLS = sql`id, state`;
export const SIBLING_OTHER = sql`name`;

async function siblingColsLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(SIBLING_COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaSiblingCols(query: SessionExecutor, id: string) {
  return siblingColsLocal(query, id);
}

async function siblingOtherLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(SIBLING_OTHER).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaSiblingOther(query: SessionExecutor, id: string) {
  return siblingOtherLocal(query, id);
}
