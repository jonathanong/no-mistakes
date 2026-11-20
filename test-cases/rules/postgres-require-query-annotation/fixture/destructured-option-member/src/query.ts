import { write, type QueryOptions } from '@example/psql'
import sql from 'sql-template-strings'

export async function objectDestructure(options: QueryOptions) {
  const run = options.query ?? write
  const { rows } = await run(sql`/* a1 */ SELECT 1`)
  return rows
}

export async function arrayDestructure(options: QueryOptions) {
  const run = options.query ?? write
  const [first] = await run(sql`/* a2 */ SELECT 1`)
  return first
}

export function nonAwaitDestructure(options: QueryOptions) {
  const run = options.query ?? write
  const { then } = run(sql`/* a3 */ SELECT 1`)
  return then
}

export async function plainString(options: QueryOptions) {
  const run = options.query ?? write
  const { rows } = await run('/* a4 */ SELECT 1')
  return rows
}

export async function plainControl(options: QueryOptions) {
  const run = options.query ?? write
  const result = await run(sql`/* c1 */ SELECT 1`)
  return result
}

export async function inlineControl(options: QueryOptions) {
  const { rows } = await (options.query ?? write)(sql`/* c2 */ SELECT 1`)
  return rows
}

export async function missingObject(options: QueryOptions) {
  const run = options.query ?? write
  const { rows } = await run(sql`SELECT 2`)
  return rows
}

export async function missingArray(options: QueryOptions) {
  const run = options.query ?? write
  const [first] = await run('SELECT 3')
  return first
}

export async function missingPlain(options: QueryOptions) {
  const run = options.query ?? write
  const result = await run(sql`SELECT 4`)
  return result
}

export async function defaultRows(options: QueryOptions) {
  const run = options.query ?? write
  const { rows = [] } = await run(sql`/* d1 */ SELECT 1`)
  return rows
}

export async function computedKey(options: QueryOptions) {
  const run = options.query ?? write
  const key = 'rows'
  const { [key]: rows } = await run(sql`/* k1 */ SELECT 1`)
  return rows
}

export async function nestedRest(options: QueryOptions) {
  const run = options.query ?? write
  const [first, ...[rest]] = await run(sql`/* r1 */ SELECT 1`)
  return first ?? rest
}

export async function nestedObject(options: QueryOptions) {
  const run = options.query ?? write
  const { data: { rows } } = await run(sql`/* n1 */ SELECT 1`)
  return rows
}

export async function identifierRest(options: QueryOptions) {
  const run = options.query ?? write
  const { rows, ...rest } = await run(sql`/* r2 */ SELECT 1`)
  return rows ?? rest
}

export async function memberCall(client: QueryOptions) {
  const { rows } = await client.query(sql`/* m1 */ SELECT 1`)
  return rows
}

export async function dynamicLeading(options: QueryOptions, x: string) {
  const run = options.query ?? write
  const { rows } = await run(sql`${x} SELECT 1`)
  return rows
}

export async function directWrite() {
  const run = write
  const { rows } = await run(sql`/* w1 */ SELECT 1`)
  return rows
}

export async function assertedQuery(options: QueryOptions) {
  const { rows } = await options.query!(sql`/* q1 */ SELECT 1`)
  return rows
}

export async function missingDefault(options: QueryOptions) {
  const run = options.query ?? write
  const { rows = [] } = await run(sql`SELECT 5`)
  return rows
}
