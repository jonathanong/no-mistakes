import sql from 'sql-template-strings'
import { read } from '@bench/db'

async function query(flag) {
  let q
  if (flag) { q = sql`/* eval */ SELECT 1` } else { q = sql`/* eval */ SELECT 1` }
  eval('opaque')
  await read(q) // low: false
}
