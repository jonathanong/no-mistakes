import sql from 'sql-template-strings'
import assert from 'node:assert'
export function appendTypeFilter(q, user, type) {
  if (!type) return
  assert(user, 401)
  q.append(sql` AND c.type = ${type}`)
}
