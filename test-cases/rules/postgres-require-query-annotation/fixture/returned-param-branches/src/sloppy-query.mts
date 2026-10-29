import { read } from '@bench/db'
import sql from 'sql-template-strings'
import { factory } from './sloppy.cjs'
export function rejected(extra) {
  const query = factory(sql`/* noSloppySource */ SELECT `, extra)
  read(query.text, query.values)
}
