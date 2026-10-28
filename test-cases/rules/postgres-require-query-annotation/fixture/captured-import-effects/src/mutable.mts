import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { append } from './effects.mjs'
// Even recovered import metadata must not excuse a write to the exact callee.
append = opaque
export function changed(items, tail) {
  const q = sql`/* changed */ SELECT 1`
  items.forEach(item => { q.append(item) })
  append(q, tail)
  read(q)
}
