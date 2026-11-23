import { query } from "@example/db";

export function archiveLine(orderId: string, lineNumber: number) {
  return query(
    `INSERT INTO order_line_archive (order_id, line_no, quantity)
     SELECT order_row.id, line.line_no, line.quantity
     FROM orders order_row
     JOIN order_lines line ON line.order_id = order_row.id
     WHERE order_row.id = $1 AND line.line_no = $2
     ON CONFLICT (order_id, line_no)
     DO UPDATE SET quantity = EXCLUDED.quantity`,
    [orderId, lineNumber],
  );
}

// Unique key on the joined table also propagates back to the parent table's primary key.
export function archiveLineFromCompositePin(
  orderId: string,
  lineNumber: number,
) {
  return query(
    `INSERT INTO order_line_archive (order_id, line_no, quantity)
     SELECT order_row.id, line.line_no, line.quantity
     FROM orders order_row
     INNER JOIN order_lines line ON order_row.id = line.order_id
     WHERE line.order_id = $1 AND line.line_no = $2
     ON CONFLICT (order_id, line_no)
     DO UPDATE SET quantity = EXCLUDED.quantity`,
    [orderId, lineNumber],
  );
}

// A non-primary unique index on orders still pins orders to one row, which pins line.order_id.
export function archiveLineByOrderCode(orderCode: string, lineNumber: number) {
  return query(
    `INSERT INTO order_line_archive (order_id, line_no, quantity)
     SELECT order_row.id, line.line_no, line.quantity
     FROM orders order_row
     JOIN order_lines line ON line.order_id = order_row.id
     WHERE order_row.code = $1 AND line.line_no = $2
     ON CONFLICT (order_id, line_no)
     DO UPDATE SET quantity = EXCLUDED.quantity`,
    [orderCode, lineNumber],
  );
}
