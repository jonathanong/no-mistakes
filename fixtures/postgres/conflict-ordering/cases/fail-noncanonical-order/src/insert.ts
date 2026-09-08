import { query } from '@data-stores/psql'

export function insert() {
  return query(`
    INSERT INTO items (tenant_id, id)
    SELECT tenant_id, id FROM pending_items ORDER BY id, tenant_id
    ON CONFLICT (tenant_id, id) DO NOTHING
  `)
}
