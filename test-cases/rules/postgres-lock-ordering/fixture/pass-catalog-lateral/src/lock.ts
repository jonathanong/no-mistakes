import { query } from "@example/db";

// The LATERAL subquery is not a lock target: only `line` is locked, and its
// ORDER BY starts with the (order_id, line_number) primary key.
export function lockOrderLines(orderIds: string[]) {
  return query(
    `SELECT line.order_id FROM order_lines line
     LEFT JOIN LATERAL (
       SELECT product.id AS product_id FROM products product
       WHERE product.sku = line.sku ORDER BY product.id LIMIT 1
     ) matched ON TRUE
     WHERE line.order_id = ANY($1::uuid[])
     ORDER BY line.order_id, line.line_number
     LIMIT 100
     FOR UPDATE OF line`,
    [orderIds],
  );
}
