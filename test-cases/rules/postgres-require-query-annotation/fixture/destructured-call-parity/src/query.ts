import { read, type TransactionQuery } from '@example/psql'
import sql from 'sql-template-strings'
import { annotatedPageQuery, bareQuery } from './builders'
import { TABLE_CONST } from './consts'

export async function templatePlain() {
  const countRows = await read(`/* warmUp */ SELECT 1 FROM ${TABLE_CONST} LIMIT 1`, [])
  return countRows
}

export async function templateDestructured() {
  const { rows: countRows } = await read(`/* warmUp */ SELECT 1 FROM ${TABLE_CONST} LIMIT 1`, [])
  return countRows
}

export async function builderPlain(after: string) {
  const result = await read(annotatedPageQuery(after, 10))
  return result
}

export async function builderDestructured(after: string) {
  const { rows } = await read(annotatedPageQuery(after, 10))
  return rows
}

export async function bareBuilderPlain(after: string) {
  const result = await read(bareQuery(after))
  return result
}

export async function bareBuilderDestructured(after: string) {
  const { rows } = await read(bareQuery(after))
  return rows
}

export async function transactionPlain(transaction: TransactionQuery) {
  const result = await transaction(sql`/* claim */ UPDATE t SET a = 1`)
  return result
}

export async function transactionDestructured(transaction: TransactionQuery) {
  const { rows } = await transaction(sql`/* claim */ UPDATE t SET a = 1`)
  return rows
}

type Run = (callback: (transaction: TransactionQuery) => Promise<unknown>) => void

export function wrappedPlain(run: Run) {
  run(async (transaction: TransactionQuery) => {
    const result = await transaction(sql`/* wrapped */ UPDATE t SET a = 1`)
    return result
  })
}

export function wrappedDestructured(run: Run) {
  run(async (transaction: TransactionQuery) => {
    const { rows } = await transaction(sql`/* wrapped */ UPDATE t SET a = 1`)
    return rows
  })
}

export async function loopPlain(ids: string[], after: string) {
  for (const id of ids) console.log(id)
  const result = await read(annotatedPageQuery(after, 10))
  return result
}

export async function loopDestructured(ids: string[], after: string) {
  for (const id of ids) console.log(id)
  const { rows } = await read(annotatedPageQuery(after, 10))
  return rows
}

export async function dynamicHeadPlain(head: string) {
  const result = await read(`${head} SELECT 1`)
  return result
}

export async function dynamicHeadDestructured(head: string) {
  const { rows } = await read(`${head} SELECT 1`)
  return rows
}
