import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { append as long } from './deep-chain/01.mts'
import { append as short } from './deep-chain/02.mts'
export function overBudget() {
  const q = sql`/* long */ SELECT 1`
  long(q)
  long(q)
  read(q)
}
export function withinBudget() {
  const q = sql`/* short */ SELECT 1`
  short(q)
  short(q)
  read(q)
}
