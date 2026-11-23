import { query } from "@example/db";

// One joined table bounded on only part of its composite primary key.
export function partialCompositeKey(orderId: string) {
  return query(
    `INSERT INTO order_line_archive (order_id, line_no, quantity)
     SELECT order_row.id, line.line_no, line.quantity
     FROM orders order_row
     JOIN order_lines line ON line.order_id = order_row.id
     WHERE order_row.id = $1
     ON CONFLICT (order_id, line_no)
     DO UPDATE SET quantity = EXCLUDED.quantity`,
    [orderId],
  );
}

// Non-unique filter column on orders leaves both tables multi-row.
export function nonUniqueFilterColumn(customerId: string, lineNumber: number) {
  return query(
    `INSERT INTO order_line_archive (order_id, line_no, quantity)
     SELECT order_row.id, line.line_no, line.quantity
     FROM orders order_row
     JOIN order_lines line ON line.order_id = order_row.id
     WHERE order_row.customer_id = $1 AND line.line_no = $2
     ON CONFLICT (order_id, line_no)
     DO UPDATE SET quantity = EXCLUDED.quantity`,
    [customerId, lineNumber],
  );
}

// Non-unique join column on shipments allows multiple shipment rows per order.
export function nonUniqueJoinColumn(orderId: string) {
  return query(
    `INSERT INTO order_line_archive (order_id, line_no, quantity)
     SELECT order_row.id, shipment.line_no, shipment.quantity
     FROM orders order_row
     JOIN shipments shipment ON shipment.order_id = order_row.id
     WHERE order_row.id = $1
     ON CONFLICT (order_id, line_no)
     DO UPDATE SET quantity = EXCLUDED.quantity`,
    [orderId],
  );
}

// LEFT JOIN is never treated as single-row.
export function leftJoin(orderId: string, lineNumber: number) {
  return query(
    `INSERT INTO order_line_archive (order_id, line_no, quantity)
     SELECT order_row.id, line.line_no, line.quantity
     FROM orders order_row
     LEFT JOIN order_lines line ON line.order_id = order_row.id
     WHERE order_row.id = $1 AND line.line_no = $2
     ON CONFLICT (order_id, line_no)
     DO UPDATE SET quantity = EXCLUDED.quantity`,
    [orderId, lineNumber],
  );
}

// ORed bounding predicate in WHERE can match multiple orders.
export function oredWherePredicate(
  firstOrderId: string,
  secondOrderId: string,
  lineNumber: number,
) {
  return query(
    `INSERT INTO order_line_archive (order_id, line_no, quantity)
     SELECT order_row.id, line.line_no, line.quantity
     FROM orders order_row
     JOIN order_lines line ON line.order_id = order_row.id
     WHERE (order_row.id = $1 OR order_row.id = $2) AND line.line_no = $3
     ON CONFLICT (order_id, line_no)
     DO UPDATE SET quantity = EXCLUDED.quantity`,
    [firstOrderId, secondOrderId, lineNumber],
  );
}

// ORed bounding predicate in ON does not pin line.order_id.
export function oredOnPredicate(orderId: string, lineNumber: number) {
  return query(
    `INSERT INTO order_line_archive (order_id, line_no, quantity)
     SELECT order_row.id, line.line_no, line.quantity
     FROM orders order_row
     JOIN order_lines line ON line.order_id = order_row.id OR line.line_no = $2
     WHERE order_row.id = $1
     ON CONFLICT (order_id, line_no)
     DO UPDATE SET quantity = EXCLUDED.quantity`,
    [orderId, lineNumber],
  );
}

// CTE shadowing a joined catalog table invalidates its unique-key proof.
export function shadowedJoinedTable(orderId: string, lineNumber: number) {
  return query(
    `WITH order_lines AS (SELECT order_id, line_no, quantity FROM staged_lines)
     INSERT INTO order_line_archive (order_id, line_no, quantity)
     SELECT order_row.id, line.line_no, line.quantity
     FROM orders order_row
     JOIN order_lines line ON line.order_id = order_row.id
     WHERE order_row.id = $1 AND line.line_no = $2
     ON CONFLICT (order_id, line_no)
     DO UPDATE SET quantity = EXCLUDED.quantity`,
    [orderId, lineNumber],
  );
}
