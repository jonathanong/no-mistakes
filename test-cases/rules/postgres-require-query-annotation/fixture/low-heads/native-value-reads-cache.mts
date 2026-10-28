import { read } from '@bench/db'
import sql from 'sql-template-strings'
function privateOwner() {
  const q = sql`/* private */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  read(q) // low: true
  read(q) // low: true
}
function escapedOwner() {
  const q = sql`/* escaped */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.at(-1))
  const alias = q
  consume(alias)
  read(q) // low: false
  read(q) // low: false
}
