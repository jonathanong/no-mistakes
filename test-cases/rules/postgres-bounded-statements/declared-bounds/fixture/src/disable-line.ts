import { query } from "@example/db";

const orphan = `SELECT 1 -- no-mistakes-bound postgres-bounded-statements rows<=100`; // no-mistakes-disable-line postgres-bounded-statements

export function orders(ids: string[]): Promise<unknown> {
  void orphan;
  void ids;
  return query("SELECT id FROM orders WHERE candidate_id = ANY($1::uuid[])");
}
