import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import { STAR_NAMED_MUT } from "./starBarrel";

async function mutatedLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(STAR_NAMED_MUT).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaStarNamedMut(query: SessionExecutor, id: string) {
  return mutatedLocal(query, id);
}
