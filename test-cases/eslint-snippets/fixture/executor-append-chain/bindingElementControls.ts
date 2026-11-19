import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

const COLS = sql`id, state`;
const { fragment = COLS } = {} as { fragment?: typeof COLS };
export function extend(dynamicFragment: unknown) {
  fragment.append(dynamicFragment);
}

async function run(query: SessionExecutor, id: string) {
  await query(sql`SELECT `.append(COLS).append(sql` FROM widgets WHERE id = ${id}`));
}

export function via(query: SessionExecutor, id: string) {
  return run(query, id);
}
