import { query } from "@example/db";

// postgres-conflict-ordering: the batch is not ordered by the conflict key.
export function insertAccounts(ids: string[], emails: string[]) {
  return query(
    `INSERT INTO accounts (id, email)
     SELECT input.id, input.email FROM unnest($1::uuid[], $2::text[]) AS input(id, email)
     ON CONFLICT (email) DO NOTHING`,
    [ids, emails],
  );
}

// postgres-lock-ordering: rows are locked in an order no index guarantees.
export function lockOrders(ids: string[]) {
  return query(`SELECT * FROM orders WHERE id = ANY($1) ORDER BY total_cents FOR UPDATE`, [ids]);
}
