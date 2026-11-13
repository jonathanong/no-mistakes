import SQL from 'sql-template-strings'
import { read } from '@example/db'

export function staticField() {
  return { query: SQL`SELECT 1` }.query
}
export function computedField() {
  return { query: SQL`SELECT 1` }['query']
}
export function appendField() {
  return { query: SQL`SELECT 1` }.query.append(' /* suffix */')
}
export function appendStatement() {
  const query = SQL`SELECT 1`
  query.append(SQL` /* suffix */`)
  return query
}
export function trueExpression() {
  return true ? { query: SQL`SELECT 1` }.query : SQL`SELECT 2`
}
export function falseExpression() {
  return false ? SQL`SELECT 2` : { query: SQL`SELECT 1` }['query']
}
export function uncertainExpression(flag) {
  return flag ? SQL`SELECT 1` : SQL`SELECT 2`
}
export function trueStatement() {
  if (true) return { query: SQL`SELECT 1` }.query
  return SQL`SELECT 2`
}
export function falseStatement() {
  if (false) return SQL`SELECT 2`
  else return { query: SQL`SELECT 1` }['query']
}
export function uncertainStatement(flag) {
  if (flag) return SQL`SELECT 1`
  else return SQL`SELECT 2`
}
function inner() { return SQL`SELECT 1` }
export function helperCall() { return inner() }
export function closedScalarCall() {
  const value = Number(1)
  return SQL`SELECT ${value}`
}
export function textConcatenation() {
  return 'SELECT ' + '1'
}
export function numericComparison(flag) {
  if (flag === 1) return SQL`SELECT 1`
  return SQL`SELECT 2`
}
export function defaultMember(value = { query: 'SELECT 1' }) {
  return value.query
}
export function defaultComputed(value = { query: 'SELECT 1' }) {
  return value['query']
}
export function interpolatedText() {
  return `SELECT ${'1'}`
}
export function optionalComputed() {
  return { query: SQL`SELECT 1` }?.['query']
}
export function aggregateArray() {
  const values = [1, 2]
  return SQL`SELECT ${values}`
}
export function logicalAnd() {
  return true && SQL`SELECT 1`
}
export function logicalOr() {
  return false || SQL`SELECT 1`
}
async function sqlPromise() { return SQL`SELECT 1` }
async function scalarPromise() { return 1 }
export function promiseDataMember() {
  const value = scalarPromise().field
  return SQL`SELECT ${value}`
}
export function promiseSqlMember() { return sqlPromise().field }
export function promiseAsData() {
  const value = Number(sqlPromise())
  return SQL`SELECT ${value}`
}
export function expressionWithoutBindings() {
  return `SELECT ${true}`
}

read(staticField())
