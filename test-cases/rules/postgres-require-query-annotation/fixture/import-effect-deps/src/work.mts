import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { appendForeign as good, appendOther as bad, absent } from './helpers.mts'
const escaped = bad
function local(q) { q.append('x') }
function positive(q, value) { local(q); good(q, value) }
function negative0(q) { bad(q) }
function negative1(q) { bad(q) }
function negative2(q) { bad(q) }
function unresolved0(q) { absent(q) }
function unresolved1(q) { absent(q) }
function run(value) {
  const a = sql`/* positive0 */ SELECT 1`; positive(a, value); read(a) // low: true
  const b = sql`/* positive1 */ SELECT 1`; positive(b, value); read(b) // low: true
  const c = sql`/* negative0 */ SELECT 1`; negative0(c); read(c) // low: false
  const d = sql`/* negative1 */ SELECT 1`; negative1(d); read(d) // low: false
  const e = sql`/* negative2 */ SELECT 1`; negative2(e); read(e) // low: false
  const f = sql`/* unresolved0 */ SELECT 1`; unresolved0(f); read(f) // low: false
  const g = sql`/* unresolved1 */ SELECT 1`; unresolved1(g); read(g) // low: false
}
