import { read } from '@bench/db'
import sql from 'sql-template-strings'
import { factory } from './other.mjs'
function run(flag) {
  const query = flag ? factory(sql`/* one */ SELECT `) : factory(sql`/* two */ SELECT `)
  read(query.text, query.values)
}
