import { read } from '@bench/db'
import sql from 'sql-template-strings'

function named(id: string) {
  const query = sql`/* namedTextField */ SELECT ${id}`
  read(query.text, query.values) // low: true
}
function namedAppend(tail: string) {
  const query = sql`/* namedAppendTextField */ SELECT 1`
  query.append(tail)
  read(query.text, query.values) // low: true
}
function missing(id: string) {
  const query = sql`SELECT ${id}`
  read(query.text, query.values) // low: false
}
function escaped() {
  const query = sql`/* escapedTextField */ SELECT 1`
  unknown(query)
  read(query.text, query.values) // low: false
}
function overwritten() {
  const query = sql`/* overwrittenTextField */ SELECT 1`
  query.text = 'SELECT 1'
  read(query.text, query.values) // low: false
}
function shadowed() {
  const query = sql`/* outerTextField */ SELECT 1`
  {
    let query = { text: 'SELECT 1', values: [] }
    read(query.text, query.values) // low: false
  }
  read(query.text, query.values) // low: true
}
function optional() {
  const query = sql`/* optionalTextField */ SELECT 1`
  read(query?.text, query.values) // low: false
  read(query['text'], query.values) // low: false
}
function fake() {
  const query = { text: '/* fake */ SELECT 1', values: [] }
  read(query.text, query.values) // low: false
}
function customTag() {
  const query = anotherTag`/* customTagField */ SELECT 1`
  read(query.text, query.values) // low: false
}

function optionsRemainSupported(options: unknown) {
  const query = sql`/* directOptionsField */ SELECT 1`
  read(query.text, query.values, options) // low: true
}

function opaqueInitializers(tail: string) {
  const named = sql`/* namedOpaqueText */ SELECT 1`.append(tail)
  read(named.text, named.values) // low: true
  const missing = sql`SELECT 1`.append(tail)
  read(missing.text, missing.values) // low: false
  // The unknown tail could turn BEGIN into BEGINNING, so EOF exemption is unsafe.
  const transaction = sql`BEGIN`.append(tail)
  read(transaction.text, transaction.values) // low: false
}
