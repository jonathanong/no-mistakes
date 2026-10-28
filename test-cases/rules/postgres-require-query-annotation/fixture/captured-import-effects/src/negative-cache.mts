import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { append } from './effects.mjs'
export function two(items, tail) {
  const q = sql`/* first */ SELECT 1`
  items.forEach(item => { q.append(item) })
  append(q, tail)
  read(q)
  const other = sql`/* second */ SELECT 1`
  items.forEach(item => { other.append(item) })
  append(other, tail)
  read(other)
}
