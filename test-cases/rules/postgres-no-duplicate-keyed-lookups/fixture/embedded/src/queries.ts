import { query } from "./db";

export async function loadOrderSummaries() {
  return query(`
    SELECT
      o.id,
      (SELECT a1.display_name FROM accounts a1 WHERE a1.id = o.account_id) AS account_name,
      (SELECT a2.tier FROM accounts a2 WHERE a2.id = o.account_id) AS account_tier
    FROM orders o
  `);
}

export async function loadOrderDetails() {
  return query(`
    SELECT o.id, a.display_name, a.tier
    FROM orders o
    LEFT JOIN accounts a ON a.id = o.account_id
  `);
}

export async function loadSuppressedOrders() {
  return query(`
    SELECT
      o.id,
      (SELECT a1.display_name FROM accounts a1 WHERE a1.id = o.account_id) AS account_name,
      -- no-mistakes-disable-next-line postgres-no-duplicate-keyed-lookups
      (SELECT a2.tier FROM accounts a2 WHERE a2.id = o.account_id) AS account_tier
    FROM orders o
  `);
}

export async function loadLineSuppressedOrders() {
  return query(`
    SELECT
      o.id,
      a1.display_name,
      a2.tier
    FROM orders o
    LEFT JOIN accounts a1 ON a1.id = o.account_id
    LEFT JOIN accounts a2 ON a2.id = o.account_id`); // no-mistakes-disable-line postgres-no-duplicate-keyed-lookups
}

export async function runDynamic(rawSql: string) {
  return query(rawSql);
}
