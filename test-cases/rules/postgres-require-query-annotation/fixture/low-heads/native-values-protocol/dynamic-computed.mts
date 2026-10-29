import { read } from '@bench/db'
import sql from 'sql-template-strings'
// Unknown bound aliases must not hide a source-visible protocol write.
const A = []['con' + 'structor']; A['pro' + 'totype'].reduce = () => 'SELECT 0'
function privateQuery() {
  const q = sql`/* privateQuery */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.at(-1))
  read(q) // low: false
  read(sql`/* directLiteral */ SELECT 1`) // low: true
}
