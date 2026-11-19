import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import { STAR_NAMED } from "./starBarrel";

async function namedLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(STAR_NAMED).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaStarNamed(query: SessionExecutor, id: string) {
  return namedLocal(query, id);
}
