import { query } from "@example/db";

// The LATERAL join is skipped only when FOR UPDATE OF names one base table and
// that table's ORDER BY is a key prefix. Each statement here must still fail.
export function wrongOrder(orderIds: string[]) {
  return query(
    `SELECT line.order_id FROM order_lines line
     LEFT JOIN LATERAL (SELECT product.id FROM products product WHERE product.sku = line.sku LIMIT 1) matched ON TRUE
     WHERE line.order_id = ANY($1)
     ORDER BY line.sku
     FOR UPDATE OF line`,
    [orderIds],
  );
}

// No OF clause: every FROM item is locked, including the lateral subquery.
export function lockEverything(orderIds: string[]) {
  return query(
    `SELECT line.order_id FROM order_lines line
     LEFT JOIN LATERAL (SELECT product.id FROM products product WHERE product.sku = line.sku LIMIT 1) matched ON TRUE
     WHERE line.order_id = ANY($1)
     ORDER BY line.order_id, line.line_number
     FOR UPDATE`,
    [orderIds],
  );
}

// OF names the derived relation itself, which is not a base table.
export function lockDerived(orderIds: string[]) {
  return query(
    `SELECT line.order_id FROM order_lines line
     LEFT JOIN LATERAL (SELECT product.id FROM products product WHERE product.sku = line.sku LIMIT 1) matched ON TRUE
     WHERE line.order_id = ANY($1)
     ORDER BY line.order_id, line.line_number
     FOR UPDATE OF matched`,
    [orderIds],
  );
}
