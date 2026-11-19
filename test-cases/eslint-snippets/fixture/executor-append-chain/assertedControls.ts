import { type SessionExecutor } from "@example/db";
import sql, { type SqlStatement } from "@example/sql";

export const ASSERTED_COLS = sql`id, state`;
export const ASSERTED_AS = sql`name`;

async function bangLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(ASSERTED_COLS!).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaAssertedBang(query: SessionExecutor, id: string) {
  return bangLocal(query, id);
}

async function asLocal(query: SessionExecutor, id: string) {
  await query(
    sql`SELECT `.append(ASSERTED_AS as SqlStatement).append(sql` FROM widgets WHERE id = ${id}`),
  );
}

export function viaAssertedAs(query: SessionExecutor, id: string) {
  return asLocal(query, id);
}
