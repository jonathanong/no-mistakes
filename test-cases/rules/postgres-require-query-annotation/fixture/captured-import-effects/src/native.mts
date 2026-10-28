import sql from 'sql-template-strings'
import { read } from '@bench/db'
export function ordinaryNative() {
  const q = sql`/* ordinaryNative */ SELECT 1`
  read(q)
}
