import { "openTransaction" as openTransaction } from '@example/db'

export async function run() {
  const tx = await openTransaction()
  await tx('BEGIN')
}
