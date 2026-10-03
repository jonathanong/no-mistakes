import { query } from "@example/db";

export function sweep(after: string) {
  return query(
    `SELECT id FROM orders WHERE id > $1 ORDER BY id LIMIT 500`,
    [after],
  );
}

export function due() {
  return query(`SELECT id FROM orders WHERE next_reconcile_at <= now() ORDER BY id LIMIT $1`, []);
}
