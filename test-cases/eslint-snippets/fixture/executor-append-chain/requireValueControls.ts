import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const REQUIRE_VALUE = sql`id, state`;
export const REQUIRE_VALUE_OTHER = sql`name`;

async function valueLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(REQUIRE_VALUE).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaRequireValue(query: SessionExecutor, id: string) {
  return valueLocal(query, id);
}

async function otherLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(REQUIRE_VALUE_OTHER).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaRequireValueOther(query: SessionExecutor, id: string) {
  return otherLocal(query, id);
}
