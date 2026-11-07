import sql from 'sql-template-strings'
import { type QueryExecutor } from 'db'
import { fragment as importedFragment } from './chain-fragment'

export function readChainedFragment() {
  const statement = sql``.append(sql`id`).append(importedFragment)
  const select = (execute: QueryExecutor) => execute(`/* chained fragment */ SELECT ${statement.text}`, statement.values)
  return select
}
