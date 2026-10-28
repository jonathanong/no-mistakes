import sql from 'sql-template-strings'
import * as templates from 'sql-template-strings'
import { read } from '@bench/db'

function other(strings, ...values) { return { strings, values } }
function wrap(value) { return value }

async function initialized(flag) {
  let q = sql`/* initialized */ SELECT 0`
  if (flag) { q = sql`/* initialized */ SELECT 1` } else { q = sql`/* initialized */ SELECT 2` }
  await read(q) // low: false
}
async function multiple(flag) {
  let q, unused
  if (flag) { q = sql`/* multiple */ SELECT 1` } else { q = sql`/* multiple */ SELECT 2` }
  await read(q) // low: false
}
async function destructured(flag) {
  let { q } = { q: undefined }
  if (flag) { q = sql`/* destructured */ SELECT 1` } else { q = sql`/* destructured */ SELECT 2` }
  await read(q) // low: false
}
async function unbraced(flag) {
  let q
  if (flag) q = sql`/* unbraced */ SELECT 1`; else q = sql`/* unbraced */ SELECT 2`
  await read(q) // low: false
}
async function memberWrite(flag) {
  let q = { text: '' }
  if (flag) { q.text = sql`/* member */ SELECT 1` } else { q.text = sql`/* member */ SELECT 2` }
  await read(q) // low: false
}
async function compound(flag) {
  let q
  if (flag) { q += sql`/* compound */ SELECT 1` } else { q += sql`/* compound */ SELECT 2` }
  await read(q) // low: false
}
async function nonTag(flag) {
  let q
  if (flag) { q = '/* non-tag */ SELECT 1' } else { q = '/* non-tag */ SELECT 2' }
  await read(q) // low: false
}
async function memberTag(flag) {
  let q
  if (flag) { q = templates.sql`/* member tag */ SELECT 1` } else { q = templates.sql`/* member tag */ SELECT 2` }
  await read(q) // low: false
}
async function customTag(flag) {
  let q
  if (flag) { q = other`/* custom tag */ SELECT 1` } else { q = other`/* custom tag */ SELECT 2` }
  await read(q) // low: false
}
async function optionalExecutor(flag) {
  let q
  if (flag) { q = sql`/* optional */ SELECT 1` } else { q = sql`/* optional */ SELECT 2` }
  await read?.(q) // low: false
}
async function extraArgument(flag) {
  let q
  if (flag) { q = sql`/* extra argument */ SELECT 1` } else { q = sql`/* extra argument */ SELECT 2` }
  await read(q, {}) // low: false
}
async function spreadArgument(flag) {
  let q
  if (flag) { q = sql`/* spread argument */ SELECT 1` } else { q = sql`/* spread argument */ SELECT 2` }
  await read(...[q]) // low: false
}
async function wrapperCall(flag) {
  let q
  if (flag) { q = sql`/* wrapper */ SELECT 1` } else { q = sql`/* wrapper */ SELECT 2` }
  read(q).then(wrap) // low: false
}
async function captured(flag) {
  function nested() { return q }
  let q
  if (flag) { q = sql`/* nested capture */ SELECT 1` } else { q = sql`/* nested capture */ SELECT 2` }
  await read(q) // low: false
  void nested
}
async function argumentsAccess(flag) {
  opaque(arguments[0])
  let q
  if (flag) { q = sql`/* arguments */ SELECT 1` } else { q = sql`/* arguments */ SELECT 2` }
  await read(q) // low: false
}
async function incompleteHead(flag) {
  let q
  if (flag) { q = sql`/* incomplete ${flag}` } else { q = sql`/* incomplete ${flag}` }
  await read(q) // low: false
}
