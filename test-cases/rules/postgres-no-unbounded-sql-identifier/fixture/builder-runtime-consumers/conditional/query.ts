import sql from 'sql-template-strings'
import { type QueryExecutor } from 'db'
import { fragment } from './producer'

const statement = sql``.append(fragment)
export const read = (execute: QueryExecutor) => execute(`SELECT ${statement.text}`, statement.values)
