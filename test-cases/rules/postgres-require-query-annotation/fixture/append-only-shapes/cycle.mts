import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { cycleA } from './cycle-a.mts'
export function cycle() {
  const q = sql`/* cycle */ SELECT 1`
  cycleA(q)
  read(q)
}
