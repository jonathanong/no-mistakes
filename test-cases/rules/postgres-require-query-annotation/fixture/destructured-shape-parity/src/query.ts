import { read, write, type QueryOptions, type TransactionQuery } from '@example/psql'
import sql from 'sql-template-strings'
import assert from 'http-assert'
import { loadWidget, recordEvent } from './barrel'
import { runInTx, captureAll } from './transactions'
import { chainedBuilder, conditionalBuilder } from './builders'

const TAIL = sql`
  id, name`

export async function optionMemberPlain(options: QueryOptions) {
  const run = options.query ?? write
  const result = await run(sql`/* multi
  */
  SELECT id FROM widgets WHERE id = ${options.id}`)
  return result
}

export async function optionMemberDestructured(options: QueryOptions) {
  const run = options.query ?? write
  const { rows } = await run(sql`/* multi
  */
  SELECT id FROM widgets WHERE id = ${options.id}`)
  return rows
}

export function transactionTailPlain() {
  return runInTx(undefined, async (transaction: TransactionQuery) => {
    const result = await transaction<{ id: string }>(
      sql`/* touch */
    UPDATE widgets SET touched = NOW()
    RETURNING `.append(TAIL),
    )
    return result
  })
}

export function transactionTailDestructured() {
  return runInTx(undefined, async (transaction: TransactionQuery) => {
    const { rows } = await transaction<{ id: string }>(
      sql`/* touch */
    UPDATE widgets SET touched = NOW()
    RETURNING `.append(TAIL),
    )
    return rows
  })
}

export async function conditionalBuilderPlain(id: string, options?: { query?: TransactionQuery }) {
  const statement = conditionalBuilder(id)
  const result = await read(statement, { query: options?.query })
  return result
}

export async function conditionalBuilderDestructured(id: string, options?: { query?: TransactionQuery }) {
  const statement = conditionalBuilder(id)
  const { rows } = await read(statement, { query: options?.query })
  return rows
}

type Capture = (label: string, run: () => Promise<void>) => Promise<void>

export async function capturedPlain(capture: Capture) {
  await capture('plain', async () => {
    const result = await read<{ id: string }>(chainedBuilder(undefined, 25))
    console.log(result)
  })
}

export async function capturedDestructured(capture: Capture) {
  await capture('destructured', async () => {
    const { rows } = await read<{ id: string }>(chainedBuilder(undefined, 25))
    console.log(rows)
  })
}

export function transactionFollowUpPlain(id: string) {
  return runInTx(undefined, async (transaction: TransactionQuery) => {
    const result = await transaction<{ id: string }>(
      sql`/* touch */
    UPDATE widgets SET touched = NOW()
    RETURNING `.append(TAIL),
    )
    const updated = result.rows[0]
    assert(updated, 404, 'missing')
    await recordEvent(id, { query: transaction })
    return loadWidget(updated.id, transaction)
  })
}

export function transactionFollowUpDestructured(id: string) {
  return runInTx(undefined, async (transaction: TransactionQuery) => {
    const { rows } = await transaction<{ id: string }>(
      sql`/* touch */
    UPDATE widgets SET touched = NOW()
    RETURNING `.append(TAIL),
    )
    const updated = rows[0]
    assert(updated, 404, 'missing')
    await recordEvent(id, { query: transaction })
    return loadWidget(updated.id, transaction)
  })
}

export async function capturedWrappedPlain() {
  await captureAll('plain', async () => {
    const result = await read<{ id: string }>(chainedBuilder(undefined, 25))
    console.log(result)
  }, undefined)
}

export async function capturedWrappedDestructured() {
  await captureAll('destructured', async () => {
    const { rows } = await read<{ id: string }>(chainedBuilder(undefined, 25))
    console.log(rows)
  }, undefined)
}
