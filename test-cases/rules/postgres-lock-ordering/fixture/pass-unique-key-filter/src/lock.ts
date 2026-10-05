import { query } from "@example/db";

// The primary key is pinned by equality, so the IN list on a non-key column
// only filters that one row and cannot make the lock multi-row.
export function lockOrder(orderId: string) {
  return query(
    `SELECT id FROM orders
     WHERE id = $1 AND status IN ('open', 'held')
     FOR UPDATE`,
    [orderId],
  );
}

// token_hash is unique; `= ANY` runs over an array column, not a key list,
// and only `t` is locked.
export function lockCallback(tokenHash: string) {
  return query(
    `SELECT t.id FROM tokens t JOIN accounts a ON a.id = t.account_id
     WHERE t.token_hash = $1 AND t.callback_url = ANY(a.callback_urls)
     FOR UPDATE OF t`,
    [tokenHash],
  );
}

// The composite primary key is fully pinned.
export function lockLine(orderId: string, lineNumber: number, skus: string[]) {
  return query(
    `SELECT order_id FROM order_lines
     WHERE order_id = $1 AND line_number = $2 AND sku = ANY($3)
     FOR UPDATE`,
    [orderId, lineNumber, skus],
  );
}
