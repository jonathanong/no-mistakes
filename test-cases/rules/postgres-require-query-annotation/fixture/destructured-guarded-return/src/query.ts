import { write, type QueryOptions } from '@example/psql'
import sql from 'sql-template-strings'
import { isId, parseId } from './helpers'

export async function typeofGuard(raw: unknown, options: QueryOptions = {}) {
  if (typeof raw !== 'string') return undefined
  const run = options.query ?? write
  const result = await run(sql`/* typeofGuard */ SELECT 1 WHERE id = ${raw}`)
  return result
}

export async function truthyGuard(raw: string | undefined, options: QueryOptions = {}) {
  if (!raw) return undefined
  const run = options.query ?? write
  const result = await run(sql`/* truthyGuard */ SELECT 1 WHERE id = ${raw}`)
  return result
}

export async function noGuard(raw: string, options: QueryOptions = {}) {
  const run = options.query ?? write
  const result = await run(sql`/* noGuard */ SELECT 1 WHERE id = ${raw}`)
  return result
}

export async function guardBareWrite(raw: unknown) {
  if (typeof raw !== 'string') return undefined
  const result = await write(sql`/* guardBareWrite */ SELECT 1 WHERE id = ${raw}`)
  return result
}

export async function guardInlineAlt(raw: unknown, options: QueryOptions = {}) {
  if (typeof raw !== 'string') return undefined
  const result = await (options.query ?? write)(sql`/* guardInlineAlt */ SELECT 1 WHERE id = ${raw}`)
  return result
}

export async function guardQueryMember(raw: unknown, options: QueryOptions = {}) {
  if (typeof raw !== 'string') return undefined
  const run = options.query
  const result = await run(sql`/* guardQueryMember */ SELECT 1 WHERE id = ${raw}`)
  return result
}

export async function importedGuard(raw: unknown, options: QueryOptions = {}): Promise<string | undefined> {
  if (typeof raw !== 'string') return undefined
  const id = parseId(raw)
  if (!id || !isId(id)) return undefined
  const run = options.query ?? write
  const { rows } = await run(sql`/* importedGuard */
    SELECT id FROM widgets WHERE id = ${id} LIMIT 1
  `)
  return rows.length > 0 ? id : undefined
}

export async function calledWithOptions(raw: unknown, options: QueryOptions) {
  const id = await importedGuard(raw, options)
  return id
}

export async function plainCallee(raw: unknown, options: QueryOptions = {}) {
  const run = options.query ?? write
  const { rows } = await run(sql`/* plainCallee */ SELECT id FROM widgets WHERE id = ${raw}`)
  return rows
}

export async function plainCaller(raw: unknown, options: QueryOptions) {
  const rows = await plainCallee(raw, options)
  return rows
}

function looksValid(value: unknown): boolean {
  return typeof value === 'string' && value.length > 0
}

export async function callGuard(raw: unknown, options: QueryOptions = {}) {
  if (typeof raw !== 'string' || !looksValid(raw)) return undefined
  const run = options.query ?? write
  const { rows } = await run(sql`/* callGuard */ SELECT 1 WHERE id = ${raw}`)
  return rows
}
