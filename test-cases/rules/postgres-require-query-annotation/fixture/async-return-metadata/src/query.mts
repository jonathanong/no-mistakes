import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { remote } from './remote.mjs'
export async function factory() {
  const statement = sql`/* factory */ SELECT 1`
  statement.append(' tail')
  return statement
}
function suffix(value) { value.append(' suffix') }
export async function withLocalEffect() {
  const withEffect = sql`/* localEffect */ SELECT 1`
  suffix(withEffect)
  return withEffect
}
const result = await remote()
read(result)
