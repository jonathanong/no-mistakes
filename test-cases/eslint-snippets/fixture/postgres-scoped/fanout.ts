import { openTransaction } from '@example/db'

export async function fanout(ids: string[]) {
  await using tx = await openTransaction()
  await Promise.all(ids.map((id) => tx('SELECT 1 WHERE id = $1', [id])))
}

// Counterintuitive on purpose: this `tx` is a parameter, not the factory result.
export async function sibling(ids: string[], tx: (sql: string, args: string[]) => unknown) {
  await Promise.all(ids.map((id) => tx('SELECT 1 WHERE id = $1', [id])))
}
