import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import { STAR_NS_MUT } from "./starBarrel";

async function mutatedLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(STAR_NS_MUT).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaStarNsMut(query: SessionExecutor, id: string) {
  return mutatedLocal(query, id);
}
