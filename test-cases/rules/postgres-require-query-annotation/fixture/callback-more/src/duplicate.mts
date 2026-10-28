import sql from 'sql-template-strings'
import { write } from '@bench/db'

// A later same-name declaration can replace the append-only helper.
function appendOnly(statement, value) { statement.append(sql`${value}`) }
function appendOnly(statement) { statement.text = 'SELECT 2' }
function source(execute) { return execute(sql`/* duplicateHelper */ SELECT 1`) }
source(statement => {
  appendOnly(statement, 1)
  return write(statement) // low: false
})
