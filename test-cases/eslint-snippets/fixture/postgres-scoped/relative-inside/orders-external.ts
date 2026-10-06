// Resolves outside the @example/db directory found by walking up from this file.
import { openTransaction } from '../relative-outside/transaction'

export async function run() {
  const tx = await openTransaction()
  await tx('BEGIN')
}
