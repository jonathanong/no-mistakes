import { read } from '@bench/db'
import { mutated } from './mutated.mjs'
export async function run() {
  const query = await mutated()
  read(query)
}
