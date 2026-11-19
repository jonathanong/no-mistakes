import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import * as parts from "./starBarrel";

async function namespaceLocal(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(parts.STAR_NS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function viaStarNamespace(query: SessionExecutor, id: string) {
  return namespaceLocal(query, id);
}
