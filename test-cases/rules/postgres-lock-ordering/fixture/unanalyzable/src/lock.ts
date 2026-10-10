import { query } from "@example/db";

// Opaque executor argument: no SQL text is recovered, so it could take a FOR UPDATE lock
// (unanalyzable, line 6).
export function opaque(sql: string, ids: string[]) {
  return query(sql, [ids]);
}

// Recovered dynamic SQL without a lock clause is treated as non-locking to keep reads quiet.
export function read(column: string) {
  return query(`SELECT id FROM jobs ORDER BY ${column}`);
}

// Recovered dynamic SQL with a lock clause keeps the ordinary ABBA check (lock-ordering, line 16).
export function lockSome(table: string, ids: string[]) {
  return query(`SELECT id FROM ${table} WHERE id = ANY($1) FOR UPDATE`, [ids]);
}

// The safe directive covers opaque SQL just like recovered SQL.
export function documented(sql: string) {
  /* deadlock-safe: callers pass single-row primary-key lookups */
  return query(sql);
}

// Line-level suppression applies to unanalyzable findings like any other finding.
export function suppressed(sql: string) {
  // no-mistakes-disable-next-line postgres-lock-ordering
  return query(sql);
}
