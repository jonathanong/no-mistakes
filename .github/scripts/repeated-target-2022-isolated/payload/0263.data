import { read } from '@example/db'
import sql from 'sql-template-strings'

function appendOriginal() {
  const query = sql`SELECT id FROM accounts`
  const alias = query
  query.append(sql` OFFSET 9`)
  return alias
}

function appendAlias() {
  const query = sql`SELECT id FROM accounts`
  const alias = query
  alias.append(sql` OFFSET 10`)
  return query
}

function chained() {
  const query = sql`SELECT id FROM accounts`
  const alias = query
  const second = alias
  second.append(sql` OFFSET 11`)
  return query
}

function reassigned() {
  let query = sql`SELECT id FROM accounts`
  const alias = query
  query = sql`SELECT id FROM accounts OFFSET 12`
  return alias
}

function branched(unknown: boolean) {
  const query = sql`SELECT id FROM accounts`
  const alias = query
  if (unknown) query.append(sql` OFFSET 13`)
  return alias
}

function escaped(unknown: (query: unknown) => void) {
  const query = sql`SELECT id FROM accounts`
  const alias = query
  unknown(query)
  return alias
}

export async function execute() {
  await read(appendOriginal())
  await read(appendAlias())
  await read(chained())
  await read(reassigned())
  await read(branched(true))
  await read(escaped(() => {}))
}
