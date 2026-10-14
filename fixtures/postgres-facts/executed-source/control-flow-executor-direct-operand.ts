import { read, sql } from '@example/db';

export const query = sql`SELECT 1`;
// The caller may mutate the exported builder before the executor runs.
export function directOpaqueOperand(mutator: () => unknown) {
  read(query, mutator());
}
