import sql from 'sql-template-strings'
import { read } from '@bench/db'

function append(config, query) { query.append(config.tail) }
function append(config, query) { query.text = config.tail }
function build(config) {
  const query = sql`/* duplicate */ SELECT 1`
  append(config, query)
  return query
}
read(build({ tail: ' WHERE x' })) // low: false
