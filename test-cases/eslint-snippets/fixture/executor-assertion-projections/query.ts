import sql from "@example/sql";
import type { SessionExecutor } from "@example/db";

export type Input = { id: string };

export function byId(query: SessionExecutor, id: string) {
  return query(sql`SELECT id FROM sample WHERE id = ${id}`);
}
