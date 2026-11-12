import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { append } from './ambiguous.mts'
export function ambiguous() {
  const q = sql`/* ambiguous */ SELECT 1`
  append(q)
  read(q)
}
