import { read, type QueryExecutor, type QueryOptions } from '@example/psql'
import sql from 'sql-template-strings'

export async function recordEvent(id: string, options?: QueryOptions) {
  const insert = sql`/* recordEvent */ INSERT INTO events (id) VALUES (${id})`
  insert.append(sql` RETURNING id`)
  return insert
}

export async function loadWidget(id: string, query: QueryExecutor = read) {
  const { rows } = await query<{ id: string }>(sql`/* loadWidget */ SELECT id FROM widgets WHERE id = ${id}`)
  return rows[0] ?? null
}
