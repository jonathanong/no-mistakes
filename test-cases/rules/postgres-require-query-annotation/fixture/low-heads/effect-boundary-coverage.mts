import { read } from '@bench/db'
import sql from 'sql-template-strings'
import { appendImported } from './effect-boundary-target.mjs'
export function appendLocal(builder) { builder.append(sql` WHERE enabled`); }
function invalidFirst({ builder }) { builder.append(sql` WHERE enabled`); }
function recoveredBody(builder) { builder.append(sql` WHERE enabled`); }
export function importedSecond() {
  const query = sql`/* named */ SELECT ${1}`
  appendImported({}, query)
  read(query) // low: false
}
