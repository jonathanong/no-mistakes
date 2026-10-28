import { read } from '@bench/db'
import { makeQuery } from './missing-factory.mjs'
export async function run() {
  const query = await makeQuery()
  read(query)
}
