import { query } from "@example/db";

// Every statement below only looks like a single-row source. Each must keep failing closed with
// missing-canonical-order (the last one with noncanonical-order).

// A set-returning function expands a FROM-less SELECT into many rows.
export function setReturning(ids: string[], tokens: string[]) {
  return query(
    `INSERT INTO receipts (order_id, token)
     SELECT unnest($1::uuid[]) AS order_id, unnest($2::text[]) AS token
     ON CONFLICT (order_id) DO NOTHING`,
    [ids, tokens],
  );
}

// Equality on a column that is not a unique key.
export function nonKeyEquality(total: number) {
  return query(
    `INSERT INTO order_archive (order_id, total)
     SELECT s.id, s.total FROM staged_orders s WHERE s.total = $1
     ON CONFLICT (order_id) DO NOTHING`,
    [total],
  );
}

// Equality on only part of a composite key.
export function partialCompositeKey(orderId: string) {
  return query(
    `INSERT INTO order_archive (order_id, total)
     SELECT l.order_id, l.total FROM order_lines l WHERE l.order_id = $1
     ON CONFLICT (order_id) DO NOTHING`,
    [orderId],
  );
}

// A disjunction matches two keys, so it is not a single pin.
export function disjunction(first: string, second: string) {
  return query(
    `INSERT INTO order_archive (order_id, total)
     SELECT o.id, o.total FROM orders o WHERE o.id = $1 OR o.id = $2
     ON CONFLICT (order_id) DO NOTHING`,
    [first, second],
  );
}

// A partial unique index only covers some rows, so equality on its column proves nothing.
export function partialUniqueIndex(id: string) {
  return query(
    `INSERT INTO order_archive (order_id, total)
     SELECT d.id, d.total FROM drafts d WHERE d.id = $1
     ON CONFLICT (order_id) DO NOTHING`,
    [id],
  );
}

// The compared value is another column, not a constant.
export function columnEquality() {
  return query(
    `INSERT INTO order_archive (order_id, total)
     SELECT o.id, o.total FROM orders o WHERE o.id = o.parent_id
     ON CONFLICT (order_id) DO NOTHING`,
    [],
  );
}

// LIMIT 2 can still return two rows.
export function limitTwo() {
  return query(
    `INSERT INTO order_archive (order_id, total)
     SELECT s.id, s.total FROM staged_orders s LIMIT 2
     ON CONFLICT (order_id) DO NOTHING`,
    [],
  );
}

// A CTE named like a catalog table can shadow it, so its primary key proves nothing.
export function shadowedByCte(id: string) {
  return query(
    `WITH orders AS (SELECT id, total FROM staged_orders)
     INSERT INTO order_archive (order_id, total)
     SELECT o.id, o.total FROM orders o WHERE o.id = $1
     ON CONFLICT (order_id) DO NOTHING`,
    [id],
  );
}

// Positional ORDER BY is mapped, so a reversed position is a reversed key order.
export function reversedPositions(accountIds: string[], userIds: string[]) {
  return query(
    `INSERT INTO members (account_id, user_id)
     SELECT input.account_id, input.user_id
     FROM unnest($1::uuid[], $2::uuid[]) AS input(account_id, user_id)
     ORDER BY 2, 1
     ON CONFLICT (account_id, user_id) DO NOTHING`,
    [accountIds, userIds],
  );
}
