import { openTransaction } from './transaction'

export async function run() {
  const tx = await openTransaction()
  await tx('BEGIN')
}
