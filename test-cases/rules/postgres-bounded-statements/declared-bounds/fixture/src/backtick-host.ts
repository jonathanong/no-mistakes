import { query } from "@example/db";

const note = "` -- no-mistakes-bound postgres-bounded-statements rows<=1: example";
// ` -- no-mistakes-bound postgres-bounded-statements rows<=1: example
/* ` -- no-mistakes-bound postgres-bounded-statements rows<=1: example */

export function orders(): Promise<unknown> {
  void note;
  return query("SELECT id FROM orders");
}
