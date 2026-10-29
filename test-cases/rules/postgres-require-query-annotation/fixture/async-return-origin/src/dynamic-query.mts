import { read } from '@bench/db'
import { dynamic } from './dynamic.mjs'
export async function run() {
  const query = await dynamic()
  read(query)
}
