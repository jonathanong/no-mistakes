import sql from 'sql-template-strings'
import { read } from '@bench/db'

function safe(config, query) { query.append(config.tail) }
function changesText(config, query) { query.text = config.tail }
function exposesConstructor(config, query) { opaque(query.constructor) }
function returnsAlias(config, query) { return query.append(config.tail) }
function capturesBuilder(config, query) { return () => query }
function spreadAppend(config, query) { query.append(...config.tail) }

function rejected(config) {
  const a = sql`/* changed */ SELECT 1`
  changesText(config, a)
  read(a) // low: false
  const b = sql`/* constructor */ SELECT 1`
  exposesConstructor(config, b)
  read(b) // low: false
  const c = sql`/* return */ SELECT 1`
  returnsAlias(config, c)
  read(c) // low: false
  const d = sql`/* capture */ SELECT 1`
  capturesBuilder(config, d)
  read(d) // low: false
  const e = sql`/* spreadAppend */ SELECT 1`
  spreadAppend(config, e)
  read(e) // low: false
  const f = sql`/* getter */ SELECT 1`
  safe({ get tail() { f.text = 'SELECT 2'; return 'x' } }, f)
  read(f) // low: false
  const g = sql`/* shadow */ SELECT 1`
  { const safe = opaque; safe(config, g) }
  read(g) // low: false
  const h = sql`/* reassigned */ SELECT 1`
  safe = opaque
  safe(config, h)
  read(h) // low: false
}
