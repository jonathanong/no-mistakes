import sql from 'sql-template-strings'
import { WINDOW_MONTHS } from './consts'

export interface Cursor {
  id: string
  createdAt: string
}

export function itemPage(after: Cursor | undefined, pageSize: number) {
  const cursor = after
    ? sql`AND (i.created_at, i.id) > (${after.createdAt}::timestamptz, ${after.id}::uuid)`
    : sql``
  return sql`/* itemPage */
    SELECT i.id, i.created_at
    FROM items i
    WHERE i.created_at <= now() - make_interval(months => ${WINDOW_MONTHS}) `.append(cursor).append(sql`
    ORDER BY i.created_at, i.id
    LIMIT ${pageSize}`)
}

export const ITEM_COLUMNS = sql`
  id, label, updated_at
`
