import { query } from "@example/db";

// A multi-target OF list is checked like the split form: no ORDER BY or SKIP
// LOCKED on a multi-row predicate is an ABBA deadlock risk.
export function lockUpdate(ids: string[]) {
  return query(`SELECT a.id, o.id FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = ANY($1) FOR UPDATE OF a, o`, [ids]);
}

export function lockNowait(ids: string[]) {
  return query(`SELECT a.id, o.id FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = ANY($1) FOR UPDATE OF a, o NOWAIT`, [ids]);
}
