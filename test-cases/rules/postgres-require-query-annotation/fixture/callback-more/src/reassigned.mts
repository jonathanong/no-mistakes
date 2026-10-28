import sql from 'sql-template-strings'
import { write } from '@bench/db'

function mutable(execute: (statement: unknown) => Promise<unknown>) {
  return execute(sql`/* mutableSource */ SELECT 1`)
}
mutable = unknown
mutable(statement => write(statement, { readOnly: false }))
