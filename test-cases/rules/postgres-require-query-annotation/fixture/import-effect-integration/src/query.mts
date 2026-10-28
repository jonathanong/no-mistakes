import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { good, nonplain, capturedTarget, returnedTarget } from './factories.mts'
import { plain, defaults, captured, returned } from './targets.mts'
function localPlain(query, value) { plain(query, value) }
function localDefaults(query, value) { defaults(query, { suffix: 'x' }) }
function localCaptured(query, value) { captured(query, value) }
function localReturned(query, value) { returned(query, value) }

async function asyncOrigins(value) {
  const a = await good(value)
  read(a) // proof: true
  const b = await nonplain(value)
  read(b) // proof: false
  const c = await capturedTarget(value)
  read(c) // proof: false
  const d = await returnedTarget(value)
  read(d) // proof: false
}
function capturedOwners(value) {
  const a = sql`/* captured/plain */ SELECT 1`
  schedule(() => { a.append(' suffix') })
  localPlain(a, value)
  read(a) // proof: true
  const b = sql`/* captured/nonplain */ SELECT 1`
  schedule(() => { b.append(' suffix') })
  localDefaults(b, value)
  read(b) // proof: false
  const c = sql`/* captured/capturedTarget */ SELECT 1`
  schedule(() => { c.append(' suffix') })
  localCaptured(c, value)
  read(c) // proof: false
  const d = sql`/* captured/returnedTarget */ SELECT 1`
  schedule(() => { d.append(' suffix') })
  localReturned(d, value)
  read(d) // proof: false
}
