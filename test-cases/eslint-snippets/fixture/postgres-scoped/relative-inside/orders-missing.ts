// ./missing does not exist, so package membership cannot be proved.
import { openTransaction } from './missing'

export async function run() {
  const tx = await openTransaction()
  await tx('BEGIN')
}
