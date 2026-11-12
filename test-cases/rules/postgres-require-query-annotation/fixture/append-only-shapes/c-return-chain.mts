import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { addOrder } from './order.mts'
export function ordering(n) {
  const q = sql`/* ordering */ SELECT 1 ORDER BY `
  addOrder(q).append(sql` LIMIT ${n}`)
  read(q)
}
