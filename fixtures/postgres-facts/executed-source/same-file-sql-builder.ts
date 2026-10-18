import { read, sql } from '@example/db'

// Same-file, non-exported builder. The sync `sql` tag with a value bind is
// the rewrite `postgres-no-async-sql-builder` reports for the async form.
function buildQuery(id: string) {
  return sql`/* orders/by-id */ SELECT id FROM orders WHERE account_id = ${id}`
}

export function loadOrders(id: string) {
  return read(buildQuery(id))
}
