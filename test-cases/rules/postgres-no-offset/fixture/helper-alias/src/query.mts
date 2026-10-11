import { read } from '@example/db'
import sql from 'sql-template-strings'

function direct() {
  const query = sql`SELECT id FROM accounts`
  query.append(sql` OFFSET 7`)
  return query
}

function aliased() {
  const query = sql`SELECT id FROM accounts`
  const alias = query
  query.append(sql` OFFSET 9`)
  return alias
}

export async function execute() {
  await read(direct())
  await read(aliased())
}
