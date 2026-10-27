import { read } from '@example/db'
import { directImportedQuery, importedQuery, suppressedImportedQuery } from './imported-source.mts'
import { otherImportedQuery } from './imported-other.mts'

export async function executeAssigned() {
  const query = importedQuery()
  await read(query)
}

export async function executeDirect() {
  await read(directImportedQuery())
  await read(suppressedImportedQuery())
  await read(otherImportedQuery())
  // Reusing one helper must still yield one finding per physical SQL star.
  await read(importedQuery())
}
