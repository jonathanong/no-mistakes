import { read } from '@bench/db'
import sql from 'sql-template-strings'
function dynamicGap() {
  const q = sql`/* gap */ SELECT 1`
  q.append(opaqueSuffix())
  consume(q.values.length)
  eval(runtimeCode)
  read(q) // low: false
}
