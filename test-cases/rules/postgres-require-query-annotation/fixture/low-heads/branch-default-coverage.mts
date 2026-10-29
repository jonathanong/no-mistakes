import sql from 'sql-template-strings'
import { read } from '@bench/db'
import other from './unknown.mjs'
function distinct(c) {
  let q
  if (c) { q = sql`/* a */ SELECT 1` } else { q = sql`/* b */ SELECT 1` }
  read(other) // low: false
}
// Bind holes keep the independent hole-free assignment proof outside this control.
function member(c, holder) {
  let q
  if (c) { holder.q = sql`/* a */ SELECT ${c}` } else { q = sql`/* b */ SELECT ${c}` }
  read(q) // low: false
}
function noEndpoint(c) {
  let q
  if (c) { q = sql`/* a */ SELECT 1` } else { q = sql`/* b */ SELECT 1` }
  return q
}
function nested(q, { value: { nested } }) { q.append(sql` suffix`) }
function overMask(q, a,b,c,d,e,f,g,h,i,j,k,l,m,n,o, { value = false } = {}) { q.append(sql` suffix`) }
