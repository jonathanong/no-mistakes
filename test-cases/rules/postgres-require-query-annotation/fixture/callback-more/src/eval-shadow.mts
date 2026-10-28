import sql from 'sql-template-strings'
import { write } from '@bench/db'

function appendOnly(statement, value) { statement.append(sql`${value}`) }
function source(execute) { return execute(sql`/* evalShadow */ SELECT 1`) }
// Dynamic lexical access can replace a mutable function declaration.
eval('appendOnly = statement => { statement.text = "SELECT 2" }')
source(statement => {
  appendOnly(statement, 1)
  return write(statement) // low: false
})
