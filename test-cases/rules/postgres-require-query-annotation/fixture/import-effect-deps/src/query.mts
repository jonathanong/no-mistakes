import sql from 'sql-template-strings'
import { read } from '@bench/db'
import defaultAppend from './helpers.mts'
import { appendForeign, appendDefaulted, appendCaptured, appendReturning, appendBroken, appendOptional, appendSpread, absent } from './helpers.mts'
import { appendForeign as reexported } from './reexport.mts'
import { appendDeeper } from './deeper.mts'
import { missing } from './missing.mts'

function local(q, user) {
  if (!user) { q.append(sql` AND public = true`); return }
  q.append(sql` AND member = ${user.id}`)
}
function combined(q, targetId, user) {
  if (user?.id === targetId) { q.append(sql`TRUE`); return }
  appendForeign(q, user)
  local(q, user)
}
function accepted(user) {
  const page = sql`/* members/page */ SELECT id FROM members WHERE `
  combined(page, 'target', user)
  page.append(sql` ORDER BY id LIMIT 20`)
  read(page) // low: true
  const count = sql`/* members/count */ SELECT count(*) FROM members WHERE `
  combined(count, 'target', user)
  read(count) // low: true
}
function defaultImported(q) { defaultAppend(q) }
function wrongPosition(q, user) { appendForeign(user, q) }
function twice(q) { appendForeign(q); appendForeign(q) }
function unknown(q) { absent(q) }
function missingSource(q) { missing(q) }
function targetDefaults(q) { appendDefaulted(q, {}) }
function targetCapture(q) { appendCaptured(q) }
function targetReturn(q) { appendReturning(q) }
function targetMutation(q) { appendBroken(q) }
function viaReexport(q) { reexported(q) }
function deeper(q) { appendDeeper(q) }
function returned(q) { appendForeign(q); return q.append('x') }
function captured(q) { appendForeign(q); schedule(() => { q.append('x') }) }
function aliased(q) { const alias = q; appendForeign(alias) }
function mutated(q) { appendForeign(q); q.text = 'SELECT 2' }
function optional(q) { appendOptional?.(q) }
function spread(q, rest) { appendSpread(q, ...rest) }
function shadowed(q, appendForeign) { appendForeign(q) }
function cycleA(q) { cycleB(q); appendForeign(q) }
function cycleB(q) { cycleA(q) }
function level0(q) { level1(q) }
function level1(q) { level2(q) }
function level2(q) { level3(q) }
function level3(q) { level4(q) }
function level4(q) { level5(q) }
function level5(q) { appendForeign(q) }
function outerDefault(q, { suffix = 'x' } = {}) { appendForeign(q, suffix) }
function rejected(user, rest, customTag) {
  const a = sql`/* multiple */ SELECT 1`; twice(a); read(a) // low: false
  const b = sql`/* unknown */ SELECT 1`; unknown(b); read(b) // low: false
  const c = sql`/* missing */ SELECT 1`; missingSource(c); read(c) // low: false
  const d = sql`/* targetDefault */ SELECT 1`; targetDefaults(d); read(d) // low: false
  const e = sql`/* targetCapture */ SELECT 1`; targetCapture(e); read(e) // low: false
  const f = sql`/* targetReturn */ SELECT 1`; targetReturn(f); read(f) // low: false
  const g = sql`/* targetMutation */ SELECT 1`; targetMutation(g); read(g) // low: false
  const h = sql`/* reexport */ SELECT 1`; viaReexport(h); read(h) // low: false
  const i = sql`/* deeper */ SELECT 1`; deeper(i); read(i) // low: false
  const j = sql`/* return */ SELECT 1`; returned(j); read(j) // low: false
  const k = sql`/* capture */ SELECT 1`; captured(k); read(k) // low: false
  const l = sql`/* alias */ SELECT 1`; aliased(l); read(l) // low: false
  const m = sql`/* mutation */ SELECT 1`; mutated(m); read(m) // low: false
  const n = sql`/* optional */ SELECT 1`; optional(n); read(n) // low: false
  const o = sql`/* spread */ SELECT 1`; spread(o, rest); read(o) // low: false
  const p = sql`/* shadow */ SELECT 1`; shadowed(p, user); read(p) // low: false
  const q = sql`/* cycle */ SELECT 1`; cycleA(q); read(q) // low: false
  const r = sql`/* depth */ SELECT 1`; level0(r); read(r) // low: false
  const s = customTag`/* custom */ SELECT 1`; combined(s, 'target', user); read(s) // low: false
  // A compatible imported target cannot excuse an incompatible outer signature.
  const t = sql`/* outerDefault */ SELECT 1`
  outerDefault(t, user)
  read(t) // low: false
  const v = sql`/* defaultImported */ SELECT 1`; defaultImported(v); read(v) // low: false
  const u = sql`/* wrongPosition */ SELECT 1`; wrongPosition(u, user); read(u) // low: false
}
