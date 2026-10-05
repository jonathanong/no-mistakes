import { query } from "@example/db";

// Without a schema catalog no key is known to be unique, so this fails closed.
export function lockOrder(orderId: string) {
  return query(
    `SELECT id FROM orders
     WHERE id = $1 AND status IN ('open', 'held')
     FOR UPDATE`,
    [orderId],
  );
}

