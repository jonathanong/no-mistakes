import { read } from '@bench/db'
import sql from 'sql-template-strings'

function named() { return sql`/* helperThirdOptions */ SELECT 1` }
function missing() { return sql`SELECT 1` }
function changed() {
  const statement = sql`/* reassignedHelper */ SELECT 1`
  statement.text = 'SELECT 1'
  return statement
}
function getter() {
  return { get text() { return '/* getter */ SELECT 1' }, values: [] }
}
function accept(options: unknown) {
  const statement = named()
  read(statement.text, statement.values, options) // low: true
}
function absent(options: unknown) {
  const statement = missing()
  read(statement.text, statement.values, options) // low: false
}
function mismatch(options: unknown) {
  const statement = named()
  const other = named()
  read(statement.text, other.values, options) // low: false
}
function fakeGetter(options: unknown) {
  const statement = getter()
  read(statement.text, statement.values, options) // low: false
}
function reassignedRoot(options: unknown) {
  const statement = changed()
  read(statement.text, statement.values, options) // low: false
}
function optionGetterCanCapture() {
  const statement = named()
  const options = { get mutate() { statement.text = 'SELECT 1'; return true } }
  read(statement.text, statement.values, options) // low: false
}
function optionalAndSpread(options: unknown) {
  const statement = named()
  read(statement?.text, statement.values, options) // low: false
  read(statement['text'], statement.values, options) // low: false
  read?.(statement.text, statement.values, options) // low: false
  read(statement.text, statement.values, ...options) // low: false
}
