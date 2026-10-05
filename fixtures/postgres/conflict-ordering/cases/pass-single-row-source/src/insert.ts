import { query } from "@example/db";

// No FROM clause: the SELECT yields at most one row.
export function recordReceipt(orderId: string, token: string) {
  return query(
    `INSERT INTO receipts (order_id, token)
     SELECT $1, $2
     WHERE EXISTS (SELECT 1 FROM orders WHERE id = $1)
     ON CONFLICT (order_id) DO UPDATE SET token = EXCLUDED.token`,
    [orderId, token],
  );
}

// One relation filtered by equality on its whole primary key yields at most one row.
export function copyOrder(orderId: string) {
  return query(
    `INSERT INTO order_archive (order_id, total)
     SELECT o.id, o.total FROM orders o WHERE o.id = $1
     ON CONFLICT (order_id) DO NOTHING`,
    [orderId],
  );
}

// A literal LIMIT 1 bounds the source whatever it selects from.
export function copyNewestStagedOrder() {
  return query(
    `INSERT INTO order_archive (order_id, total)
     SELECT s.id, s.total FROM staged_orders s LIMIT 1
     ON CONFLICT (order_id) DO NOTHING`,
    [],
  );
}

// A composite key is fully pinned by equalities on every key column.
export function copyOrderLine(orderId: string, lineNo: number) {
  return query(
    `INSERT INTO order_archive (order_id, total)
     SELECT l.order_id, l.total FROM order_lines l WHERE l.order_id = $1 AND l.line_no = $2::int
     ON CONFLICT (order_id) DO NOTHING`,
    [orderId, lineNo],
  );
}
