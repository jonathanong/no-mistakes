import sql from 'sql-template-strings'
import { read } from '@example/db'
import { appendUnknown } from './opaque-append.ts'

export function opaqueRead(runtimeSql: string) {
  const query = sql`/* opaque */ WITH eligible AS (SELECT id FROM posts) `
  appendUnknown(query, runtimeSql)
  query.append(sql`SELECT id FROM eligible`)
  return read(query)
}
