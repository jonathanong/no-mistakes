import { read } from '@bench/db'
import sql from 'sql-template-strings'

// Observations may run code and return objects; only the SQL head is proved.
async function recipients(flag: boolean, bound: unknown, offset: number) {
  const q = sql`/* recipients */ SELECT ${bound}`
  q.append(sql` WHERE active = ${flag}`)
  q.append(opaqueSuffix())
  q.append(sql` AND field0 = ${bound}`)
  q.append(sql` AND field1 = ${bound}`)
  q.append(sql` AND field2 = ${bound}`)
  q.append(sql` AND field3 = ${bound}`)
  q.append(sql` AND field4 = ${bound}`)
  q.append(sql` AND field5 = ${bound}`)
  q.append(sql` AND field6 = ${bound}`)
  q.append(sql` AND field7 = ${bound}`)
  const start = q.values.length
  q.append(sql` LIMIT ${offset}`)
  observe(q.values[start], q.values[start + 1], q.values.at(-1))
  read(q) // low: true
  read(q) // low: true
}
function arbitraryIndex(index: unknown) {
  const q = sql`/* index */ SELECT ${unknownObject}`
  q.append(opaqueSuffix())
  consume(q.values[index])
  read(q) // low: true
}
function arbitraryAt() {
  const q = sql`/* at */ SELECT ${unknownObject}`
  q.append(opaqueSuffix())
  consume(q.values.at(unknownOffset))
  read(q) // low: true
}
