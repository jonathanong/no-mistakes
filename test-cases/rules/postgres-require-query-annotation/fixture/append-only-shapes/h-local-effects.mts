import sql from 'sql-template-strings'
import { read } from '@bench/db'
function addFilters(q) { q.append(sql` WHERE true`) }
function rewrite(q) { q.strings[0] = 'SELECT 2' }
export function conditionalLocal(flag) {
  const q = flag ? sql`/* first */ SELECT 1` : sql`/* second */ SELECT 2`
  addFilters(q)
  read(q) // low: true
}
export function conditionalOpaque(flag) {
  const q = flag ? sql`/* first */ SELECT 1` : sql`/* second */ SELECT 2`
  opaque(q)
  read(q) // low: false
}
export function conditionalRewrite(flag) {
  const q = flag ? sql`/* first */ SELECT 1` : sql`/* second */ SELECT 2`
  rewrite(q)
  read(q) // low: false
}
