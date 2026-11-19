import type { QueryOptions } from '@example/psql'

declare function next(): string

export async function guardAssignsScalar(options: QueryOptions) {
  let text = '/* annotated */ SELECT 1'
  if ((text = next()) === 'skip') return undefined
  const { rows } = await options.query(text)
  return rows
}

export async function guardAssignsNested(options: QueryOptions) {
  let text = '/* annotated */ SELECT 1'
  if (next() === 'a' && (text = next()) === 'skip') return undefined
  await options.query(text)
}

export async function guardReadsOnly(options: QueryOptions) {
  const text = '/* annotated */ SELECT 1'
  if (next() === 'skip') return undefined
  await options.query(text)
}
