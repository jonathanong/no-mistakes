import { query } from "@example/db";

// An unresolved name in the OF list fails closed instead of checking only the
// resolved relation.
export function lockPair(ids: string[]) {
  return query(`SELECT * FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = ANY($1) ORDER BY id FOR UPDATE OF a, missing`, [ids]);
}
