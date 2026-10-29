import sql from 'sql-template-strings'
import { read } from '@bench/db'

function suffix(config, query) {
  if (!config.tail) return
  query.append(config.tail)
}
function more(config, query) { suffix(config, query); query.append(' ') }
function alias(config, query) { const leaked = query; leaked.append(config.tail) }
function used(config, query) { const leaked = query.append(config.tail); opaque(leaked) }
function returned(config, query) { return query }
function exposed(config, query) { opaque(query) }
function badDefault(config = {}, query) { query.append(config.tail) }
function captured(config, query) { return () => query.append(config.tail) }

function named(config) {
  const query = sql`/* electionRows */ SELECT 1`
  more(config, query)
  return query
}
function missing(config) {
  const query = sql`SELECT 1`
  suffix(config, query)
  return query
}
function namedWithRuntimeSuffix(config) {
  const query = sql`/* electionStats */ SELECT 1`
  suffix(config, query)
  query.append(config.tail)
  return query
}
function custom(config) {
  const query = other`/* custom */ SELECT 1`
  suffix(config, query)
  return query
}

read(named({ tail: ' WHERE x' })) // low: true
read(missing({ tail: ' WHERE x' })) // low: false
read(namedWithRuntimeSuffix({ tail: ' WHERE x' })) // low: true
read(custom({ tail: ' WHERE x' })) // low: false
read(sql`SELECT 1`) // low: false

function rejected(config) {
  const a = sql`/* alias */ SELECT 1`
  alias(config, a)
  read(a) // low: false
  const b = sql`/* used */ SELECT 1`
  used(config, b)
  read(b) // low: false
  const c = sql`/* returned */ SELECT 1`
  returned(config, c)
  read(c) // low: false
  const d = sql`/* exposed */ SELECT 1`
  exposed(config, d)
  read(d) // low: false
  const e = sql`/* default */ SELECT 1`
  badDefault(config, e)
  read(e) // low: false
  const f = sql`/* captured */ SELECT 1`
  captured(config, f)
  read(f) // low: false
  const g = sql`/* argument */ SELECT 1`
  suffix(opaque(g), g)
  read(g) // low: false
  const h = sql`/* optional */ SELECT 1`
  suffix?.(config, h)
  read(h) // low: false
  const i = sql`/* spread */ SELECT 1`
  suffix(...[config, i])
  read(i) // low: false
  const j = sql`/* usedCaller */ SELECT 1`
  const leaked = suffix(config, j)
  read(j) // low: false
}
