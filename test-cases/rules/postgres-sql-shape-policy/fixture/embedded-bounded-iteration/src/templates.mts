import sql from 'sql-template-strings'
import { query } from '@example/db'

export function pages(after: string, size: number) {
  return query(sql`SELECT id FROM orders WHERE id > ${after} ORDER BY id LIMIT ${size}`)
}
