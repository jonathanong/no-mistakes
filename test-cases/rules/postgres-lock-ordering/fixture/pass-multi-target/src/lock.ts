import { query } from "@example/db";

// One locking clause may list several relations after OF; each FOR UPDATE / FOR SHARE
// form and wait policy below must stay parseable and ordered.
export function lockUpdate(ids: string[]) {
  return query(`SELECT a.id, o.id FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = ANY($1) ORDER BY a.id, o.id FOR UPDATE OF a, o`, [ids]);
}

export function lockShare(ids: string[]) {
  return query(`SELECT a.id, o.id FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = ANY($1) ORDER BY a.id, o.id FOR SHARE OF a, o`, [ids]);
}

export function lockNowait(ids: string[]) {
  return query(`SELECT a.id, o.id FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = ANY($1) ORDER BY a.id, o.id FOR UPDATE OF a, o NOWAIT`, [ids]);
}

export function lockSkipLocked(ids: string[]) {
  return query(`SELECT a.id, o.id FROM accounts a JOIN orders o ON o.account_id = a.id WHERE a.id = ANY($1) FOR UPDATE OF a, o SKIP LOCKED`, [ids]);
}
