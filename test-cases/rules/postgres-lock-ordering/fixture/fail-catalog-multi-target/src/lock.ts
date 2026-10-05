import { query } from "@example/db";

// The catalog key prefix is required for every relation in the OF list: the
// order starts with accounts' key but not orders' key.
export function lockPair(ids: string[]) {
  return query(`SELECT * FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = ANY($1) ORDER BY a.id FOR UPDATE OF a, o`, [ids]);
}
