import { query } from "@example/db";

export function pinned(id: string) {
  return query(
    `INSERT INTO order_archive (order_id, total) SELECT o.id, o.total FROM orders o WHERE o.id = $1 ON CONFLICT (order_id) DO NOTHING`,
    [id],
  );
}
