import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { append } from './effects.mjs'
export function gap(items, tail) {
  const q = sql`/* gap */ SELECT 1`
  items.forEach(item => { q.append(item) })
  append(q, eval('tail'))
  read(q)
}
