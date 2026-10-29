import SQL from 'sql-template-strings'
import { read } from '@example/db'

function filtered(purpose: string) {
  const query = SQL`SELECT id FROM placements WHERE deleted_at IS NULL`
  if (purpose !== 'retain') query.append(SQL` AND live = true`)
  if (purpose !== 'inform') query.append(SQL` AND responders = true`)
  if (purpose === 'inform') query.append(SQL` AND owner = true`)
  if (purpose === 'retain') query.append(SQL` AND retained = true`)
  return query
}

function dangerous(purpose: string) {
  if (purpose !== 'retain') return SQL`SELECT id FROM placements`
  return SQL`DROP TABLE placements`
}

function coercive(purpose: string) {
  const query = SQL`SELECT id FROM placements`
  if (purpose != 'retain') query.append(SQL` AND live = true`)
  if (purpose != 'inform') query.append(SQL` AND responders = true`)
  if (purpose == 'inform') query.append(SQL` AND owner = true`)
  if (purpose == 'retain') query.append(SQL` AND retained = true`)
  return query
}

export function examples(purpose: string) {
  read(filtered(purpose)); // CORRELATED: retain, inform, other are three complete variants.
  read(dangerous(purpose)); // KEEP_DROP: a known strict branch still reaches DROP TABLE.
  read(coercive(purpose)); // COERCIVE: loose equality does not refine the alternatives.
}
