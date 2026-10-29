import { read } from '@bench/db'
import sql from 'sql-template-strings'

function deferred(tail: string, options: unknown) {
  const query = sql`/* deferredText */ SELECT 1`
  const appendLater = () => { query.append(tail) }
  const execute = () => read(query.text, query.values, options) // low: true
  appendLater()
  return execute
}
function replayed(tail: string) {
  const query = sql`/* replayedText */ SELECT 1`
  const execute = () => read(query.text, query.values) // low: true
  const later = () => { query.append(tail) }
  later()
  execute()
  later()
  execute()
}
function alias() {
  const query = sql`/* aliasText */ SELECT 1`
  const other = query
  const execute = () => read(query.text, query.values) // low: false
  return other && execute
}
function write() {
  const query = sql`/* writeText */ SELECT 1`
  const later = () => { query.text = 'SELECT 1' }
  const execute = () => read(query.text, query.values) // low: false
  return later && execute
}
function unknownMethod() {
  const query = sql`/* methodText */ SELECT 1`
  const later = () => { query.reset() }
  const execute = () => read(query.text, query.values) // low: false
  return later && execute
}
function missingPair() {
  const query = sql`/* missingPair */ SELECT 1`
  const execute = () => read(query.text, []) // low: false
  return execute
}
function optionCaptures() {
  const query = sql`/* optionCapture */ SELECT 1`
  const options = { get value() { query.text = 'SELECT 1'; return true } }
  const execute = () => read(query.text, query.values, options) // low: false
  return execute
}
function customTag() {
  const query = customSql`/* customText */ SELECT 1`
  const execute = () => read(query.text, query.values) // low: false
  return execute
}
