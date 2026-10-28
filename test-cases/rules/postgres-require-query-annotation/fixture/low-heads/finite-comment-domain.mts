import { read } from '@bench/db'
import sql from 'sql-template-strings'

function aliasedHelper(name: string) {
  const query = sql``.append(`/* aliasedHelper:${name} */ SELECT 1`)
  read(query) // low: false
}
function aliasCaller() {
  const select = (name: string) => aliasedHelper(name)
  const alias = aliasedHelper
  select('known')
  void alias
}
function literalArgument(name: string) {
  const query = sql``.append(`/* literalArgument:${name} */ SELECT 1`)
  read(query) // low: false
}
function literalCaller() {
  const select = (name: string) => literalArgument('fixed')
  select('known')
}
function mutableParameter(name: string) {
  const query = sql``.append(`/* mutableParameter:${name} */ SELECT 1`)
  read(query) // low: false
}
function mutableCaller() {
  const select = (name: string) => { name = runtimeName; return mutableParameter(name) }
  select('known')
}
function mutableSelector(name: string) {
  const query = sql``.append(`/* mutableSelector:${name} */ SELECT 1`)
  read(query) // low: false
}
function selectorCaller() {
  const select = (name: string) => mutableSelector(name)
  select = otherSelector
  select('known')
}
function emptyLiteral(name: string) {
  const query = sql``.append(`/* emptyLiteral:${name} */ SELECT 1`)
  read(query) // low: false
}
function emptyCaller() {
  const select = (name: string) => emptyLiteral(name)
  select('')
}
export function exportedHelper(name: string) {
  const query = sql``.append(`/* exportedHelper:${name} */ SELECT 1`)
  read(query) // low: false
}
function exportCaller() {
  const select = (name: string) => exportedHelper(name)
  select('known')
}
function unusedRoot(name: string) {
  const query = sql``.append(`/* ${name} */ SELECT 1`)
  void query
}
function unusedCaller() {
  const select = (name: string) => unusedRoot(name)
  select('known')
}
function closedEarly(name: string) {
  const query = sql``.append(`/* ${name} */ SELECT 1`)
  read(query) // low: false
}
function closedCaller() {
  const select = (name: string) => closedEarly(name)
  select('*/')
}
