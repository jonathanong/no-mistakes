import sql from 'sql-template-strings'
import { read } from '@bench/db'

function appendSuffix(builder, suffix) { builder.append(suffix) }
function appendNested(builder, suffix) { appendSuffix(builder, suffix) }
function escape(builder) { opaque(builder) }

function named(suffix) {
  const statement = sql`/* stages/load */ SELECT 1`
  appendNested(statement, suffix)
  read(statement)
}
function missing(suffix) {
  const statement = sql`SELECT 1`
  appendNested(statement, suffix)
  read(statement)
}
function fields(suffix) {
  const statement = sql`SELECT 1`
  appendSuffix(statement, suffix)
  read(statement.text, statement.values)
}
function partial(suffix) {
  const statement = sql`/* `
  appendSuffix(statement, suffix)
  read(statement)
}
function transaction(suffix) {
  const statement = sql`BEGIN`
  appendSuffix(statement, suffix)
  read(statement)
}
function escaped(suffix) {
  const statement = sql`SELECT 1`
  escape(statement)
  appendSuffix(statement, suffix)
  read(statement)
}
function unknownTag(suffix) {
  const statement = custom`SELECT 1`
  appendSuffix(statement, suffix)
  read(statement)
}
function nestedFields(suffix) {
  const statement = sql`SELECT 1`
  appendNested(statement, suffix)
  read(statement.text, statement.values)
}
