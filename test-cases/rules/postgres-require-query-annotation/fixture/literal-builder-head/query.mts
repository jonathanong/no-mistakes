import sql, { type SQLStatement } from 'sql-template-strings'
import { read } from '@bench/db'
import { addExternalFilters } from '@bench/builders'
import { addImportedFilters, replaceImportedHead } from './helpers.mts'

function addFilters(q: SQLStatement, id: string) { q.append(sql` AND x = ${id}`) }
export async function f(id: string) {
  const query = sql`/* listThings */ SELECT id FROM things WHERE true`
  addFilters(query, id)
  query.append(sql` LIMIT 10`)
  return read(query)
}
function unknownEscape(id) {
  const query = sql`/* escapedThings */ SELECT * FROM things`
  opaque(query, id)
  query.append(sql` LIMIT 10`)
  read(query)
}
function reassigned(id) {
  let query = sql`/* reassigned */ SELECT 1`
  addFilters(query, id)
  query = other
  read(query)
}
function dynamicHead(head, id) {
  const query = sql`${head} SELECT 1`
  addFilters(query, id)
  read(query)
}
function unsupportedMethod(id) {
  const query = sql`/* replacing */ SELECT 1`
  addFilters(query, id)
  query.setName('other')
  read(query)
}
function replacingField(id) {
  const query = sql`/* replacing */ SELECT 1`
  addFilters(query, id)
  query.strings = ['SELECT 2']
  read(query)
}
function replacingApi(id) {
  const query = sql`/* replacing */ SELECT 1`
  addFilters(query, id)
  Object.assign(query, { strings: ['SELECT 2'] })
  read(query)
}
function customTag(id) {
  const query = custom`/* custom */ SELECT 1`
  opaque(query, id)
  read(query)
}
function replaceHead(builder) { builder.strings[0] = 'SELECT 2' }
function replacingHelper(id) {
  const query = sql`/* replacing */ SELECT 1`
  replaceHead(query)
  query.append(sql` LIMIT 10`)
  read(query)
}
function resetHead(builder) { builder.reset() }
function replacingHelperMethod(id) {
  const query = sql`/* replacing */ SELECT 1`
  resetHead(query)
  query.append(sql` LIMIT 10`)
  read(query)
}
function missingHead(id) {
  const query = sql`SELECT 1`
  addFilters(query, id)
  query.append(sql` LIMIT 10`)
  read(query)
}
function externalHead(id) {
  const query = sql`/* externalThings */ SELECT 1`
  addExternalFilters(query, id)
  query.append(sql` LIMIT 10`)
  read(query)
}
function importedHead(id) {
  const query = sql`/* importedThings */ SELECT 1`
  addImportedFilters(query, id)
  query.append(sql` LIMIT 10`)
  read(query)
}
function importedMutation(id) {
  const query = sql`/* importedMutation */ SELECT 1`
  replaceImportedHead(query)
  query.append(sql` LIMIT 10`)
  read(query)
}
