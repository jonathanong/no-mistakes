import { read } from '@bench/db'
import { gap } from './gap.mjs'
export async function run() {
  const query = await gap()
  read(query)
}
