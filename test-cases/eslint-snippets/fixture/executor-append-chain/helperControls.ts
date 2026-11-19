import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";
import { MUTATED, PARAMETERIZED } from "./fragments";

export async function mutatedHelper(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(MUTATED).append(sql` FROM widgets WHERE id = ${id}`));
}

export async function parameterHelper(
  query: SessionExecutor,
  id: string,
  part: ReturnType<typeof sql>,
) {
  await query(sql`SELECT `.append(part).append(sql` FROM widgets WHERE id = ${id}`));
}

export async function parameterizedHelper(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(PARAMETERIZED).append(sql` FROM widgets WHERE id = ${id}`));
}
