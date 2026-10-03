import { query } from '@example/db'

export function insert() {
  return query(`
    INSERT INTO items (tenant_id, id)
    SELECT tenant_id, id FROM pending_items ORDER BY id, tenant_id
    ON CONFLICT (tenant_id, id) DO NOTHING
  `)
}
