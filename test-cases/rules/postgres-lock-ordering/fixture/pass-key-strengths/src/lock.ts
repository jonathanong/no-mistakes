import { query } from "@example/db";

// FOR NO KEY UPDATE is checked like FOR UPDATE: ordered locks pass, and SKIP
// LOCKED remains an alternative to ordering.
export function lockOrdered(ids: string[]) {
  return query(`SELECT a.id, o.id FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = ANY($1) ORDER BY a.id, o.id FOR NO KEY UPDATE OF a, o`, [ids]);
}

export function lockSingle(ids: string[]) {
  return query(`SELECT * FROM accounts WHERE id = ANY($1) ORDER BY id FOR NO KEY UPDATE`, [ids]);
}

export function lockSkipLocked(ids: string[]) {
  return query(`SELECT * FROM accounts WHERE id = ANY($1) FOR NO KEY UPDATE SKIP LOCKED`, [ids]);
}

// FOR KEY SHARE is a shared lock. Like FOR SHARE it is deliberately not
// checked, so the missing ORDER BY here must not be reported.
export function shareUnordered(ids: string[]) {
  return query(`SELECT a.id FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = ANY($1) FOR KEY SHARE OF a, o`, [ids]);
}
