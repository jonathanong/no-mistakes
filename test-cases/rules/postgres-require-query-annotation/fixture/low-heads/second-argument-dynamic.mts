import sql from 'sql-template-strings'
import { read } from '@bench/db'

function append(config, query) { query.append(config.tail) }
function build(config) {
  const query = sql`/* dynamic */ SELECT 1`
  eval('query.text = "SELECT 2"')
  append(config, query)
  return query
}
read(build({ tail: ' WHERE x' })) // low: false
