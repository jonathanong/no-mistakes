import sql from 'sql-template-strings'
import { type QueryExecutor } from 'db'
import { buildRead, buildLocal, statement as importedStatement, fragment } from './builders'

export function readFactory(ids: string[]) {
  const statement = buildRead(ids)
  const select = (execute: QueryExecutor) => execute(`/* factory */ SELECT ${statement.text}`, statement.values)
  return select
}

export const readImported = (execute: QueryExecutor) => execute(`/* imported */ SELECT ${importedStatement.text}`, importedStatement.values)

export function readLocal(id: string) {
  const statement = buildLocal(id)
  const select = (execute: QueryExecutor) => execute(`/* local result */ SELECT ${statement.text}`, statement.values)
  return select
}

export function readFragment(id: string) {
  const statement = sql``.append(fragment).append(sql` FROM items WHERE id = ${id}`)
  const select = (execute: QueryExecutor) => execute(`/* fragment */ SELECT ${statement.text}`, statement.values)
  return select
}
