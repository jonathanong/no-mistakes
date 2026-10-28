import sql from 'sql-template-strings'
import { read } from '@bench/db'

function execute(tail: string) {
  const query = sql`/* fieldMaxParity */ SELECT 1`
  read(query.text, query.values)
  query.append(tail)
  read(query)
}
