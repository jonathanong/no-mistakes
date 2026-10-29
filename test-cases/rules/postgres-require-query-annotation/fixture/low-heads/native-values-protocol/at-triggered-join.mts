import { read } from '@bench/db'
import sql from 'sql-template-strings'
// This module violates/exposes the native protocol; no purity of .at is assumed.
Array.prototype.at = function () { Array.prototype.join = () => 'SELECT 0'; return this[0] }
function privateQuery() {
  const q = sql`/* privateQuery */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.at(-1))
  read(q) // low: false
  read(sql`/* directLiteral */ SELECT 1`) // low: true
}
