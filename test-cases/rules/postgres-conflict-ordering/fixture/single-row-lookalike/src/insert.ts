import { query } from "@example/db";

export function lookalike(total: number) {
  return query(
    `INSERT INTO order_archive (order_id, total) SELECT o.id, o.total FROM orders o WHERE o.total = $1 ON CONFLICT (order_id) DO NOTHING`,
    [total],
  );
}
