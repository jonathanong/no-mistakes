import sql from 'sql-template-strings'
import custom from 'custom-sql'
import { filters, order, unsafe, cycle } from './effects.mjs'
import { unknown } from './missing.mjs'

export async function named(user, options) {
  const { enabled } = options
  const rows = await setup(user)
  const query = sql`/* asyncOrigin */ SELECT ${rows}`.append(tableName).append(sql` WHERE user_id = ${user}`)
  query.append(sql` AND active`)
  filters(query, { enabled })
  order(query, 10)
  return query
}
export { named as namedAlias }
export async function missing() {
  const query = sql`SELECT `
  query.append(sql` 1`)
  return query
}

export async function early() {
  if (flag) return sql`/* other */ SELECT 1`
  const query = sql`/* early */ SELECT 1`
  return query
}

export async function finallyOverride() {
  try { const query = sql`/* finally */ SELECT 1`; return query } finally { return other() }
}

export async function postAwait() {
  const query = sql`/* await */ SELECT 1`
  query.append(await suffix())
  return query
}

export async function escape() {
  const query = sql`/* escape */ SELECT 1`
  unsafe(query)
  return query
}

export async function unresolved() {
  const query = sql`/* unknown */ SELECT 1`
  unknown(query)
  return query
}

export async function cyclic() {
  const query = sql`/* cycle */ SELECT 1`
  cycle(query)
  return query
}

export async function alias() {
  const query = sql`/* alias */ SELECT 1`
  const alias = query
  return query
}

export async function captured() {
  const query = sql`/* captured */ SELECT 1`
  register(() => query.append(sql` suffix`))
  return query
}

export async function customTag() {
  const query = custom`/* custom */ SELECT 1`
  return query
}

export async function begin() {
  const query = sql`BEGIN`
  query.append('NING SELECT 1')
  return query
}

export async function comment() {
  const query = sql`/*`
  query.append(' fixed */ SELECT 1')
  return query
}

export async function reassigned() {
  const query = sql`/* reassigned */ SELECT 1`
  query = other()
  return query
}

export async function defaulted(options = mutate()) {
  const query = sql`/* defaulted */ SELECT 1`
  return query
}

export async function deep() {
  const query = sql`/* deep */ SELECT 1`.append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`).append(sql` suffix`)
  return query
}

export async function budget() {
  const query = sql`/* budget */ SELECT 1`
  order(query, 1)
  order(query, 1)
  order(query, 1)
  order(query, 1)
  order(query, 1)
  order(query, 1)
  order(query, 1)
  order(query, 1)
  order(query, 1)
  return query
}

export async function getterEscape() {
  const query = sql`/* getter */ SELECT 1`
  filters(query, { get enabled() { query.text = 'SELECT unsafe'; return true } })
  return query
}
export async function secondPosition() {
  const query = sql`/* position */ SELECT 1`
  otherPosition(null, query)
  return query
}

// Disposal runs after return and is outside sequential preallocation setup.
export async function disposal() {
  using resource = acquire()
  const query = sql`/* disposal */ SELECT 1`
  return query
}

// Even an await within the root initializer suspends a partially built heap.
export async function initializerAwait() {
  const query = sql`/* initializerAwait */ SELECT 1`.append(await suffix())
  return query
}
export async function taggedAwait() {
  const query = sql`/* taggedAwait */ SELECT ${await value()}`
  return query
}
