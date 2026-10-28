import { read } from '@bench/db'
import sql from 'sql-template-strings'

function wholeArray() {
  const q = sql`/* wholeArray */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  consume(q.values)
  read(q) // low: false
}
function arrayAlias() {
  const q = sql`/* arrayAlias */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  const values = q.values; consume(values)
  read(q) // low: false
}
function unknownProperty() {
  const q = sql`/* unknownProperty */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  consume(q.values.custom)
  read(q) // low: false
}
function unknownMethod() {
  const q = sql`/* unknownMethod */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  q.values.map(callback)
  read(q) // low: false
}
function capture() {
  const q = sql`/* capture */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  const get = () => q.values.length; consume(get)
  read(q) // low: false
}
function queryAlias() {
  const q = sql`/* queryAlias */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  const alias = q; consume(alias)
  read(q) // low: false
}
function queryEscape() {
  const q = sql`/* queryEscape */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  consume(q)
  read(q) // low: false
}
function indexCapture() {
  const q = sql`/* indexCapture */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  consume(q.values[{ toString() { q.text = "SELECT 0"; return "0" } }])
  read(q) // low: false
}
function boundCapture() {
  const q = sql`/* boundCapture */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  q.append(sql`${() => q}`)
  read(q) // low: false
}
function selfBinding() {
  const q = sql`/* selfBinding */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  q.append(sql`${q}`)
  read(q) // low: false
}
function propertyWrite() {
  const q = sql`/* propertyWrite */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  q.values[0].field = 1
  read(q) // low: false
}
function indexWrite() {
  const q = sql`/* indexWrite */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  q.values[0] = 1
  read(q) // low: false
}
function lengthWrite() {
  const q = sql`/* lengthWrite */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  q.values.length = 0
  read(q) // low: false
}
function deleteValue() {
  const q = sql`/* deleteValue */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  delete q.values[0]
  read(q) // low: false
}
function updateValue() {
  const q = sql`/* updateValue */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  q.values[0]++
  read(q) // low: false
}
function optional() {
  const q = sql`/* optional */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  consume(q?.values.length)
  read(q) // low: false
}
function optionalAt() {
  const q = sql`/* optionalAt */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  consume(q.values.at?.(-1))
  read(q) // low: false
}
function spreadAt() {
  const q = sql`/* spreadAt */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  consume(q.values.at(...args))
  read(q) // low: false
}
function emptyAt() {
  const q = sql`/* emptyAt */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  consume(q.values.at())
  read(q) // low: false
}
function twoAt() {
  const q = sql`/* twoAt */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  consume(q.values.at(1, 2))
  read(q) // low: false
}
function computedCall() {
  const q = sql`/* computedCall */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  q.values[0]()
  read(q) // low: false
}
function extractedAt() {
  const q = sql`/* extractedAt */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  const at = q.values.at; at(-1)
  read(q) // low: false
}
function stringsMutation() {
  const q = sql`/* stringsMutation */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  q.strings[0] = "SELECT 0"
  read(q) // low: false
}
function customProperty() {
  const q = sql`/* customProperty */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  q.text = "SELECT 0"
  read(q) // low: false
}
function queryReturn() {
  const q = sql`/* queryReturn */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  return q
  read(q) // low: false
}
function partialHead() {
  const q = sql`BEGIN`
  q.append('NING')
  consume(q.values.length)
  read(q) // low: false
}
function openComment() {
  const q = sql`/* partial`
  q.append('*/ SELECT 1')
  consume(q.values.length)
  read(q) // low: false
}
function customTag() {
  const q = custom`/* custom */ SELECT ${bound}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  read(q) // low: false
}
function initializerCapture() {
  const q = sql`/* capture */ SELECT ${() => q}`
  q.append(opaqueSuffix())
  consume(q.values.length)
  read(q) // low: false
}
