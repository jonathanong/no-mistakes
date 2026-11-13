import sql from 'sql-template-strings'
import { read } from '@bench/db'
function addX(q, b) { if (b) q.append(sql` AND x = ${b}`) }
function addY(q, b) { q.append(sql` AND y = ${b}`) }
function addFilters(q, b) {
  if (!b) { q.append(sql`TRUE`); return }
  addX(q, b)
  addY(q, b)
}
export function local(b) {
  const q = sql`/* local */ SELECT 1 WHERE `
  addFilters(q, b)
  read(q)
}
