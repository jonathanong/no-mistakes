import sql from 'sql-template-strings'
import { write } from '@bench/db'

function appendOnly(statement, value) { statement.append(sql`${value}`) }
function source(execute) { return execute(sql`/* shadowedHelper */ SELECT 1`) }
function run(appendOnly) {
  source(statement => {
    appendOnly(statement, 1)
    return write(statement) // low: false
  })
}
