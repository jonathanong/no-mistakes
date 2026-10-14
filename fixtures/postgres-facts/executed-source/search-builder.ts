import sql from 'sql-template-strings'
import { appendSearchCte } from './search-append.ts'

export function buildSearch() {
  const query = sql`/* search/list */ `
  appendSearchCte(query)
  query.append(sql`SELECT id FROM eligible`)
  return query
}
