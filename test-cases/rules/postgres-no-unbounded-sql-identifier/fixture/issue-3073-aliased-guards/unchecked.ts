import { beginTransaction } from 'db'
type Allowed = 'alpha' | 'beta' | 'gamma'

export async function uncheckedUnion(tableName: Allowed, items: string[]) {
  if (items.length === 0) return []
  await using query = await beginTransaction()
  return await query(
    `/* uncheckedUnion */ SELECT id FROM ${tableName} WHERE id = ANY($1::uuid[]) ORDER BY id FOR UPDATE`,
    [items],
  )
}
