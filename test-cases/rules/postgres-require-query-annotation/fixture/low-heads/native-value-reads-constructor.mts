import { read } from '@bench/db'
import sql from 'sql-template-strings'

// Constructor escape is module-wide even when the mutation runs elsewhere.
sql.SQLStatement.prototype.text = 'SELECT 0'
function changedConstructor() {
  const q = sql`/* changed */ SELECT 1`
  q.append(opaqueSuffix())
  consume(q.values.length)
  read(q) // low: false
}
