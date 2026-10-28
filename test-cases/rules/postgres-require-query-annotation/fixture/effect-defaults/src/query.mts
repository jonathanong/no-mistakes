import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { report, topics, plainDestructure, widePlain, badFirst, badIdentifier, badProperty, badOuter, badComputed, badProto } from './helpers.mts'
import { unavailable } from './missing.mts'

function localReport(q, { active = false } = {}) {
  q.append(active ? ' AND active' : ' AND pending')
}
function accepted(ids, suffix) {
  const a = sql`/* defaults/report */ SELECT 1`
  report(a, { includeAvailable: true })
  read(a) // low: true
  const b = sql`/* defaults/omitted */ SELECT 1`
  report(b)
  read(b) // low: true
  const c = sql`/* defaults/topics */ SELECT 1`
  topics(c, ids, 'posts')
  read(c) // low: true
  const d = sql`/* defaults/string */ SELECT 1`
  topics(d, ids)
  read(d) // low: true
  const e = sql`/* defaults/plain */ SELECT 1`
  plainDestructure(e, { suffix })
  read(e) // low: true
  const f = sql`/* defaults/widePlain */ SELECT 1`
  widePlain(f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, { suffix })
  read(f) // low: true
  const g = sql`/* defaults/local */ SELECT 1`
  localReport(g, { active: true })
  read(g) // low: true
}
function rejected(ids, options) {
  const a = sql`/* defaults/spread */ SELECT 1`
  report(a, { ...options })
  read(a) // low: false
  const b = sql`/* defaults/getter */ SELECT 1`
  report(b, { get includeAvailable() { return true } })
  read(b) // low: false
  const c = sql`/* defaults/method */ SELECT 1`
  report(c, { includeAvailable() { return true } })
  read(c) // low: false
  const d = sql`/* defaults/call */ SELECT 1`
  report(d, { includeAvailable: opaque() })
  read(d) // low: false
  const e = sql`/* defaults/computed */ SELECT 1`
  report(e, { [options.key]: true })
  read(e) // low: false
  const f = sql`/* defaults/proto */ SELECT 1`
  report(f, { __proto__: options })
  read(f) // low: false
  const g = sql`/* defaults/first */ SELECT 1`
  badFirst(g)
  read(g) // low: false
  const h = sql`/* defaults/identifier */ SELECT 1`
  badIdentifier(h)
  read(h) // low: false
  const i = sql`/* defaults/property */ SELECT 1`
  badProperty(i)
  read(i) // low: false
  const j = sql`/* defaults/outer */ SELECT 1`
  badOuter(j)
  read(j) // low: false
  const k = sql`/* defaults/pattern */ SELECT 1`
  badComputed(k, { key: true })
  read(k) // low: false
  const l = sql`/* defaults/protoPattern */ SELECT 1`
  badProto(l)
  read(l) // low: false
  const m = sql`/* defaults/undefined */ SELECT 1`
  report(m, undefined)
  read(m) // low: false
  const n = sql`/* defaults/unresolved */ SELECT 1`
  unavailable(n, { includeAvailable: true })
  read(n) // low: false
  const o = sql`/* defaults/alias */ SELECT 1`
  const alias = o
  report(o, { includeAvailable: true })
  read(o) // low: false
  const p = sql`/* defaults/escape */ SELECT 1`
  report(p, { get includeAvailable() { p.text = 'SELECT 2'; return true } })
  read(p) // low: false
  const q = sql`/* defaults/argumentEscape */ SELECT 1`
  report(q, { includeAvailable: opaque(q) })
  read(q) // low: false
}
