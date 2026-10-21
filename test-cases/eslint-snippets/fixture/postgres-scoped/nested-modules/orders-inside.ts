import { openTransaction } from './node_modules/@example/db/transaction'

export async function run() {
  const tx = await openTransaction()
  await tx('BEGIN')
}
