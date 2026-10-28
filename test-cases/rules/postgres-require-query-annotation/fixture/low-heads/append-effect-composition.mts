import sql from 'sql-template-strings'
import { read } from '@bench/db'

function leaf(q, suffix) { q.append(suffix) }
function middle(q, suffix) { leaf(q, suffix); q.append(' ') }
function top(q, suffix) { middle(q, suffix) }
function returning(q, suffix) { return q.append(suffix).append(' ') }
function returnedDestructure(q, { suffix }) { return q.append(suffix) }
function conditional(q, suffix) {
  if (suffix) return q.append(suffix)
  q.append(' ')
}
function usedNested(q, suffix) { const alias = leaf(q, suffix); opaque(alias) }
function captured(q, suffix) { schedule(() => { leaf(q, suffix) }) }
function aliased(q, suffix) { const alias = q; leaf(alias, suffix) }
function selfSuffix(q) { return q.append(opaque(q)) }
function wrongReturn(q) { return q }
function computedSecondary(q, { [runtimeKey]: suffix }) { q.append(suffix) }
function defaultedSecondary(q, { suffix = opaque(q) } = {}) { q.append(suffix) }
function cycleA(q) { cycleB(q) }
function cycleB(q) { cycleA(q) }

function accepted(suffix) {
  const q = sql`/* composed */ SELECT 1`
  top(q, suffix)
  returning(q, suffix)
  returnedDestructure(q, { suffix })
  conditional(q, suffix)
  read(q) // low: true
}
function rejected(suffix) {
  const a = sql`/* used */ SELECT 1`
  usedNested(a, suffix)
  read(a) // low: false
  const b = sql`/* captured */ SELECT 1`
  captured(b, suffix)
  read(b) // low: false
  const c = sql`/* aliased */ SELECT 1`
  aliased(c, suffix)
  read(c) // low: false
  const d = sql`/* selfSuffix */ SELECT 1`
  selfSuffix(d)
  read(d) // low: false
  const e = sql`/* wrongReturn */ SELECT 1`
  wrongReturn(e)
  read(e) // low: false
  const f = sql`/* cycle */ SELECT 1`
  cycleA(f)
  read(f) // low: false
  const g = sql`/* usedCaller */ SELECT 1`
  const alias = returning(g, suffix)
  read(g) // low: false
  const h = sql`/* computed */ SELECT 1`
  computedSecondary(h, { suffix })
  read(h) // low: false
  const i = sql`/* default */ SELECT 1`
  defaultedSecondary(i)
  read(i) // low: false
  const j = sql`/* getter */ SELECT 1`
  returnedDestructure(j, { get suffix() { j.text = 'SELECT 2'; return 'x' } })
  read(j) // low: false
  const k = sql`/* argument */ SELECT 1`
  returnedDestructure(k, { suffix: opaque(k) })
  read(k) // low: false
}
