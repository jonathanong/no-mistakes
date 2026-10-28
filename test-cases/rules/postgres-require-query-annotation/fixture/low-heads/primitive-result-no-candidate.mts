import sql from 'sql-template-strings'
import { read } from '@bench/db'

function named(type) {
  return `/* readOne */ SELECT ${type}`
}
function ordinary(type) {
  const unconsumed = named(type)
  read(sql`/* ordinary */ SELECT 1`) // low: true
  return unconsumed
}
