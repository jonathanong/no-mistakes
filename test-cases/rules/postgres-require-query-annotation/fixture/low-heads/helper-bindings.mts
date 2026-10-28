import { read } from '@bench/db'
import sql from 'sql-template-strings'

function named(value: unknown) { return sql`/* helperBinding */ SELECT ${value}` }
function missing() { return sql`SELECT 1` }
function chained() { return sql`/* chainedHelper */ SELECT 1`.append(dynamicTail) }
function customTag() { return customSql`/* customTagHelper */ SELECT 1` }
function plain() { return { text: '/* plain */ SELECT 1', values: [] } }
function stringHead() { return '/* string */ SELECT 1' }
function returnedLocal(value: unknown) {
  const statement = sql`/* returnedLocal */ SELECT ${value}`
  statement.append(sql` WHERE true`)
  return statement
}
function returnedEscaped() {
  const statement = sql`/* returnedEscaped */ SELECT 1`
  unknown(statement)
  return statement
}
function changed() { return sql`/* changedHelper */ SELECT 1` }
changed = missing

function uses(value: unknown) {
  const a = named(value)
  read(a) // low: true
  const b = missing()
  read(b) // low: false
  const c = returnedLocal(value)
  c.append(sql` WHERE true`)
  read(c) // low: true
  const chainedResult = chained()
  read(chainedResult) // low: true
  const untrustedResult = customTag()
  read(untrustedResult) // low: false
  const d = named(value).append(sql` WHERE true`)
  read(d) // low: true
  const returnedUnknown = returnedEscaped()
  read(returnedUnknown) // low: false
  read(named(value).append(sql` WHERE true`)) // low: true
  read(missing().append(sql` WHERE true`)) // low: false
  read(plain().append(sql` WHERE true`)) // low: false
  const plainValue = plain()
  read(plainValue) // low: false
  const stringValue = stringHead()
  read(stringValue) // low: false
  const e = named(value)
  unknown(e)
  read(e) // low: false
  const f = named(value)
  const alias = f
  read(f) // low: false
  const g = named(value)
  g.text = 'SELECT 1'
  read(g) // low: false
  const h = named(value)
  const used = h.append(sql` WHERE true`)
  read(h) // low: false
  const i = changed()
  read(i) // low: false
  const j = named?.(value)
  read(j) // low: false
  const k = named(...args)
  read(k) // low: false
  const l = named(value)
  const capture = () => l
  read(l) // low: false
  for (const item of [1, 2]) {
    const m = named(item)
    read(m) // low: false
  }
}
