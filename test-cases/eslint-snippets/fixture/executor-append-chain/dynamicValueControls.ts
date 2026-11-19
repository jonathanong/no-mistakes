import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const DYNAMIC_VALUE = sql`id, state`;
export const DYNAMIC_VALUE_OTHER = sql`name`;

async function valueLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(DYNAMIC_VALUE).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaDynamicValue(query: SessionExecutor, id: string) {
  return valueLocal(query, id);
}

async function otherLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(DYNAMIC_VALUE_OTHER).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaDynamicValueOther(query: SessionExecutor, id: string) {
  return otherLocal(query, id);
}
