import { beginTransaction } from 'db'
import { ALLOWED_TABLES } from '@scope/pkg/config'
type Allowed = (typeof ALLOWED_TABLES)[number]

export async function lockRows(tableName: Allowed, items: string[]) {
  if (items.length === 0) return []
  if (!ALLOWED_TABLES.includes(tableName)) {
    throw new Error(`Invalid table name: ${tableName}`)
  }
  await using query = await beginTransaction()
  return await query(
    `/* lockRows */ SELECT id FROM ${tableName} WHERE id = ANY($1::uuid[]) ORDER BY id FOR UPDATE`,
    [items],
  )
}
