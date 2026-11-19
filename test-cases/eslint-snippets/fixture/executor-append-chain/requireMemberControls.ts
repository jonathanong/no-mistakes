import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export const REQUIRE_MEMBER = sql`id, state`;
export const REQUIRE_KEEP = sql`name`;

async function memberLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(REQUIRE_MEMBER).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaRequireMember(query: SessionExecutor, id: string) {
  return memberLocal(query, id);
}

async function keepLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(REQUIRE_KEEP).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaRequireKeep(query: SessionExecutor, id: string) {
  return keepLocal(query, id);
}
