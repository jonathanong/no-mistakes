import { query } from "@example/db";

// Both locked relations are keyed by their id column, so the unqualified id
// order is a catalog key prefix for each resolved relation.
export function lockPair(ids: string[]) {
  return query(`SELECT * FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = ANY($1) ORDER BY id FOR UPDATE OF a, o`, [ids]);
}
