import { read, beginTransaction } from '@example/db'
import { buildSearch } from './search-builder.ts'
import { buildCount } from './search-count.ts'

export async function listSearch() {
  const query = buildSearch()
  return read(query)
}

export async function countSearch() {
  const query = buildSearch()
  return read(buildCount(query))
}

export async function semanticSearch() {
  const query = buildSearch()
  await using transaction = await beginTransaction()
  const result = await transaction(query)
  await transaction.commit()
  return result
}
