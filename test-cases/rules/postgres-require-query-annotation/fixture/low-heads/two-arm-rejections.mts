import sql from 'sql-template-strings'
import * as templates from 'sql-template-strings'
import { read } from '@bench/db'

// Bind holes keep the older mutable-literal proof out of these controls.

function other(strings, ...values) { return { strings, values } }
function wrap(value) { return value }

async function initialized(flag) {
  let q = sql`/* initialized */ SELECT ${flag}`
  if (flag) { q = sql`/* initialized */ SELECT ${flag}` } else { q = sql`/* initialized */ SELECT ${flag}` }
  await read(q) // low: false
}
async function multiple(flag) {
  let q, unused
  if (flag) { q = sql`/* multiple */ SELECT ${flag}` } else { q = sql`/* multiple */ SELECT ${flag}` }
  await read(q) // low: false
}
async function destructured(flag) {
  let { q } = { q: undefined }
  if (flag) { q = sql`/* destructured */ SELECT ${flag}` } else { q = sql`/* destructured */ SELECT ${flag}` }
  await read(q) // low: false
}
async function unbraced(flag) {
  let q
  if (flag) q = sql`/* unbraced */ SELECT ${flag}`; else q = sql`/* unbraced */ SELECT ${flag}`
  await read(q) // low: false
}
async function memberWrite(flag) {
  let q = { text: '' }
  if (flag) { q.text = sql`/* member */ SELECT ${flag}` } else { q.text = sql`/* member */ SELECT ${flag}` }
  await read(q) // low: false
}
async function compound(flag) {
  let q
  if (flag) { q += sql`/* compound */ SELECT ${flag}` } else { q += sql`/* compound */ SELECT ${flag}` }
  await read(q) // low: false
}
async function nonTag(flag) {
  let q
  if (flag) { q = `/* non-tag */ SELECT ${flag}` } else { q = `/* non-tag */ SELECT ${flag}` }
  await read(q) // low: false
}
async function memberTag(flag) {
  let q
  if (flag) { q = templates.sql`/* member tag */ SELECT ${flag}` } else { q = templates.sql`/* member tag */ SELECT ${flag}` }
  await read(q) // low: false
}
async function customTag(flag) {
  let q
  if (flag) { q = other`/* custom tag */ SELECT ${flag}` } else { q = other`/* custom tag */ SELECT ${flag}` }
  await read(q) // low: false
}
async function optionalExecutor(flag) {
  let q
  if (flag) { q = sql`/* optional */ SELECT ${flag}` } else { q = sql`/* optional */ SELECT ${flag}` }
  await read?.(q) // low: false
}
async function extraArgument(flag) {
  let q
  if (flag) { q = sql`/* extra argument */ SELECT ${flag}` } else { q = sql`/* extra argument */ SELECT ${flag}` }
  await read(q, {}) // low: false
}
async function spreadArgument(flag) {
  let q
  if (flag) { q = sql`/* spread argument */ SELECT ${flag}` } else { q = sql`/* spread argument */ SELECT ${flag}` }
  await read(...[q]) // low: false
}
async function wrapperCall(flag) {
  let q
  if (flag) { q = sql`/* wrapper */ SELECT ${flag}` } else { q = sql`/* wrapper */ SELECT ${flag}` }
  read(q).then(wrap) // low: false
}
async function captured(flag) {
  function nested() { return q }
  let q
  if (flag) { q = sql`/* nested capture */ SELECT ${flag}` } else { q = sql`/* nested capture */ SELECT ${flag}` }
  await read(q) // low: false
  void nested
}
async function argumentsAccess(flag) {
  opaque(arguments[0])
  let argumentQuery
  if (flag) { argumentQuery = sql`/* arguments */ SELECT ${flag}` } else { argumentQuery = sql`/* arguments */ SELECT ${flag}` }
  await read(argumentQuery) // low: false
}
async function incompleteHead(flag) {
  let q
  if (flag) { q = sql`/* incomplete ${flag}` } else { q = sql`/* incomplete ${flag}` }
  await read(q) // low: false
}
