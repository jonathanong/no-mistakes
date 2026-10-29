import { read } from '@bench/db'
// The awaited imported call is real, but its following sink is not configured.
export async function unused() {
  const result = await read('/* unrelatedAwait */ SELECT 1')
  consume(result)
}
