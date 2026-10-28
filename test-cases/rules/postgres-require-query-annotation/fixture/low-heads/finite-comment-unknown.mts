import { read } from '@bench/db'
import sql from 'sql-template-strings'

const aliasedRows = [['one', 'SELECT 1']] as const
for (const [name, body] of aliasedRows) {
  read(`/* ${name} */ ${body}`) // low: false
}
for (const [name, body] of [[...aliasedRows[0]]] as const) {
  read(`/* ${name} */ ${body}`) // low: false
}
for (let [name, body] of [['mutable', 'SELECT 1']] as const) {
  read(`/* ${name} */ ${body}`) // low: false
}
for (const [name = 'default', body] of [['provided', 'SELECT 1']] as const) {
  read(`/* ${name} */ ${body}`) // low: false
}
for (const [name, body] of [['outer', 'SELECT 1']] as const) {
  const callback = (name: string) => read(`/* ${name} */ ${body}`) // low: false
  callback('inner')
}
for (const [name, body] of [['valid', 'SELECT 1']] as const) {
  read(`/* ${name} ${unknown} */ ${body}`) // low: false
  read(`/* ${name} missing close ${body}`) // low: false
}
for (const [name, body] of [['', 'SELECT 1']] as const) {
  read(`/* ${name} */ ${body}`) // low: false
}

async function extraCall() {
  const select = (name: string) => extraRows(name)
  select('known')
  extraRows(runtimeName)
}
function extraRows(name: string) {
  const query = sql``.append(`/* extra:${name} */ SELECT 1`)
  read(query) // low: false
}
async function dynamicCall() {
  const select = (name: string) => dynamicRows(name)
  select(runtimeName)
}
function dynamicRows(name: string) {
  const query = sql``.append(`/* dynamic:${name} */ SELECT 1`)
  read(query) // low: false
}
async function escapedSelect() {
  const select = (name: string) => escapedRows(name)
  const alias = select
  alias('known')
}
function escapedRows(name: string) {
  const query = sql``.append(`/* escaped:${name} */ SELECT 1`)
  read(query) // low: false
}
async function wrongTag() {
  const select = (name: string) => untrustedRows(name)
  select('known')
}
function untrustedRows(name: string) {
  const query = custom``.append(`/* untrusted:${name} */ SELECT 1`)
  read(query) // low: false
}
async function optionalCall() {
  const select = (name: string) => optionalRows(name)
  select?.('known')
}
function optionalRows(name: string) {
  const query = sql``.append(`/* optional:${name} */ SELECT 1`)
  read(query) // low: false
}
async function spreadCall() {
  const select = (name: string) => spreadRows(name)
  select(...['known'])
}
function spreadRows(name: string) {
  const query = sql``.append(`/* spread:${name} */ SELECT 1`)
  read(query) // low: false
}
async function escapedRoot() {
  const select = (name: string) => escapedRootRows(name)
  select('known')
}
function escapedRootRows(name: string) {
  const query = sql``.append(`/* escapedRoot:${name} */ SELECT 1`)
  opaque(query)
  read(query) // low: false
}
async function taggedInterpolation() {
  const select = (name: string) => taggedRows(name)
  select('known')
}
function taggedRows(name: string) {
  // A tag binds interpolation values; they are not SQL comment bytes.
  const query = sql`/* tagged:${name} */ SELECT 1`
  read(query) // low: false
}
