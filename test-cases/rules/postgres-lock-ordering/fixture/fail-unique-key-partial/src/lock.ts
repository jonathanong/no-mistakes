import { query } from "@example/db";

// Each statement below still locks more than one row. Do not "fix" the rule by
// accepting them.
export function nonKeyEquality(status: string) {
  return query(`SELECT id FROM orders WHERE status = $1 AND id IN ($2, $3) FOR UPDATE`, [status, 1, 2]);
}

// Only part of the composite key is pinned.
export function partialCompositeKey(orderId: string, skus: string[]) {
  return query(`SELECT order_id FROM order_lines WHERE order_id = $1 AND sku = ANY($2) FOR UPDATE`, [orderId, skus]);
}

// The key equality sits inside an OR, so it does not bound the result.
export function pinInsideOr(orderId: string, ids: string[]) {
  return query(`SELECT id FROM orders WHERE (id = $1 OR status = 'open') AND status = ANY($2) FOR UPDATE`, [orderId, ids]);
}

// The pinned key belongs to a table that is not the locked one.
export function pinOnOtherTable(accountId: string, ids: string[]) {
  return query(
    `SELECT o.id FROM orders o JOIN accounts a ON a.id = o.account_id
     WHERE a.id = $1 AND o.status = ANY($2) FOR UPDATE OF o`,
    [accountId, ids],
  );
}

// Both tables are locked and only one is pinned.
export function lockBothPinOne(orderId: string, statuses: string[]) {
  return query(
    `SELECT o.id FROM orders o JOIN accounts a ON a.id = o.account_id
     WHERE o.id = $1 AND a.status = ANY($2) FOR UPDATE`,
    [orderId, statuses],
  );
}

// An unqualified column is ambiguous across two relations, so it pins nothing.
export function unqualifiedAcrossJoin(orderId: string, statuses: string[]) {
  return query(
    `SELECT o.id FROM orders o JOIN accounts a ON a.id = o.account_id
     WHERE id = $1 AND o.status = ANY($2) FOR UPDATE OF o`,
    [orderId, statuses],
  );
}

// The key is compared to a column of another row, not a bound value.
export function keyEqualsColumn(statuses: string[]) {
  return query(
    `SELECT o.id FROM orders o JOIN accounts a ON a.id = o.account_id
     WHERE o.id = a.order_id AND o.status = ANY($1) FOR UPDATE OF o`,
    [statuses],
  );
}
