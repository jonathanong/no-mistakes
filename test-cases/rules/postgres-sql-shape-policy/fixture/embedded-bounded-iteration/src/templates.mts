import sql from 'sql-template-strings'
import { query } from '@example/db'

export function pages(after: string, size: number) {
  return query(sql`SELECT id FROM orders WHERE id > ${after} ORDER BY id LIMIT ${size}`)
}

export function literalMarker(size: number) {
  return query(sql`SELECT id FROM orders WHERE sql_placeholder_1 = true AND id > $1 ORDER BY id LIMIT ${size}`)
}

export function markerAndInterpolation(after: string, size: number) {
  return query(sql`SELECT id FROM orders WHERE sql_placeholder_1 = true AND id > ${after} ORDER BY id LIMIT ${size}`)
}
