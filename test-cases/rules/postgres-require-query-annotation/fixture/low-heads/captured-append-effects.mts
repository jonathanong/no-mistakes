import sql from 'sql-template-strings'
import { read } from '@bench/db'

function appendEach(q, parts) {
  parts.forEach((part, index) => {
    if (index) q.append(', ')
    q.append(part)
  })
}
function appendQueued(q, parts) {
  const callbacks = parts.map(part => () => { q.append(part) })
  callbacks.forEach(callback => schedule(callback))
  callbacks.forEach(callback => schedule(callback))
}
function appendConditional(q, parts) {
  parts.forEach(part => { if (part) q.append(part) })
}
function returnBuilder(q) { schedule(() => { return q }) }
function returnAppend(q, part) { schedule(() => q.append(part)) }
function returnAppendBlock(q, part) { schedule(() => { return q.append(part) }) }
function aliasBuilder(q) { schedule(() => { const alias = q; alias.append('x') }) }
function writeText(q) { schedule(() => { q.text = 'SELECT 2' }) }
function unknownMethod(q) { schedule(() => { q.reset() }) }
function argumentEscape(q) { schedule(() => { opaque(q) }) }
function selfSuffix(q) { schedule(() => { q.append(opaque(q)) }) }
function defaultCapture(q) {
  schedule((part = opaque(q)) => { q.append(part) })
}
function appendLeaf(q, part) { q.append(part) }
function nestedEffect(q, part) { schedule(() => { appendLeaf(q, part) }) }

function accepted(parts) {
  const each = sql`/* captured/each */ SELECT 1`
  appendEach(each, parts)
  read(each) // low: true
  const queued = sql`/* captured/queued */ SELECT 1`
  appendQueued(queued, parts)
  read(queued) // low: true
  const conditional = sql`/* captured/conditional */ SELECT 1`
  appendConditional(conditional, parts)
  read(conditional) // low: true
}
function rejected(part) {
  const returned = sql`/* captured/return */ SELECT 1`
  returnBuilder(returned)
  read(returned) // low: false
  const arrow = sql`/* captured/arrow */ SELECT 1`
  returnAppend(arrow, part)
  read(arrow) // low: false
  const returnedAppend = sql`/* captured/returned-append */ SELECT 1`
  returnAppendBlock(returnedAppend, part)
  read(returnedAppend) // low: false
  const alias = sql`/* captured/alias */ SELECT 1`
  aliasBuilder(alias)
  read(alias) // low: false
  const write = sql`/* captured/write */ SELECT 1`
  writeText(write)
  read(write) // low: false
  const method = sql`/* captured/method */ SELECT 1`
  unknownMethod(method)
  read(method) // low: false
  const argument = sql`/* captured/argument */ SELECT 1`
  argumentEscape(argument)
  read(argument) // low: false
  const suffix = sql`/* captured/suffix */ SELECT 1`
  selfSuffix(suffix)
  read(suffix) // low: false
  const defaulted = sql`/* captured/default */ SELECT 1`
  defaultCapture(defaulted)
  read(defaulted) // low: false
  const nested = sql`/* captured/effect */ SELECT 1`
  nestedEffect(nested, part)
  read(nested) // low: false
}
