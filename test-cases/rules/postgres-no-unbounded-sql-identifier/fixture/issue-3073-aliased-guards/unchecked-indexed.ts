import { beginTransaction } from 'db'
import { ALLOWED_TABLES } from '@scope/pkg/config'
type Allowed = (typeof ALLOWED_TABLES)[number]

export async function uncheckedIndexedUnion(tableName: Allowed, items: string[]) {
  if (items.length === 0) return []
  await using query = await beginTransaction()
  return await query(
    `/* uncheckedIndexedUnion */ SELECT id FROM ${tableName} WHERE id = ANY($1::uuid[]) ORDER BY id FOR UPDATE`,
    [items],
  )
}
