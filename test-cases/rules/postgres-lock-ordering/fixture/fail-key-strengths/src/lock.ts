import { query } from "@example/db";

// These statements use only FOR NO KEY UPDATE, so they used to skip the rule
// entirely. A multi-row predicate without ORDER BY or SKIP LOCKED is an ABBA
// deadlock risk, for a single target and for an OF list.
export function lockSingle(ids: string[]) {
  return query(`SELECT * FROM accounts WHERE id = ANY($1) FOR NO KEY UPDATE`, [ids]);
}

export function lockOfList(ids: string[]) {
  return query(`SELECT a.id, o.id FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = ANY($1) FOR NO KEY UPDATE OF a, o`, [ids]);
}
