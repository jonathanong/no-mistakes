import { query } from "@example/db";

export function byFlag(flag: boolean): Promise<unknown> {
  return query(flag ? `SELECT id FROM orders WHERE candidate_id = ANY($1::uuid[])` : `SELECT id FROM orders WHERE candidate_id = ANY($2::uuid[])`); // no-mistakes-bound postgres-bounded-statements rows<=40: two recovered statements share this line
}
