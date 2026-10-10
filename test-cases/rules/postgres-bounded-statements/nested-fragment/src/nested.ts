import { query, sql } from "@example/db";

// The nested fragment is spliced SQL, not a bind value. Reading it as a bind
// turned `FROM accounts ${...}` into `FROM accounts sql_placeholder_1` (an
// alias), so the rule judged an unbounded SELECT it never actually saw.
export function load(id: string, includeAll: boolean) {
  return query(sql`SELECT id FROM accounts ${includeAll ? sql`` : sql`WHERE id = ${id}`}`);
}
