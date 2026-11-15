import { query } from "@example/db";

const hidden = "-- no-mistakes-bound postgres-bounded-statements rows<=1: hidden in a string";

export function page(ids: string[]): Promise<unknown> {
  void hidden;
  void ids;
  return query(`
    -- no-mistakes-bound postgres-bounded-statements rows<=25: caller passes one page of ids
    SELECT id FROM orders WHERE candidate_id = ANY($1::uuid[])
  `);
}

export function host(ids: string[]): Promise<unknown> {
  // no-mistakes-bound postgres-bounded-statements rows<=30: caller passes one page of ids
  return query(`SELECT id FROM orders WHERE candidate_id = ANY($1::uuid[])`, [ids]);
}
