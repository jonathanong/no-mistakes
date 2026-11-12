import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { appendDueSource } from './i-due-source.mts'
import { mutatingPage, opaquePage, pageHelper, replacingPage } from './i-page-helper.mts'
import { unknownPage } from './i-unknown-page.mts'
export async function searchDue(options, now) {
  const query = sql`/* searchDue */ WITH page AS MATERIALIZED (SELECT DISTINCT d.id`
  appendDueSource(query, now)
  return pageHelper(options, 'msg', 'name', 'kind', query, async statement => {
    statement.append(sql`) SELECT 1 FROM page`)
    appendDueSource(statement, now)
    return read(statement)
  })
}
export async function opaqueHelper(options, now) {
  const query = sql`/* opaque */ SELECT 1`
  return opaquePage(options, 'msg', 'name', 'kind', query, async statement => {
    return read(statement)
  })
}
export async function mutatingHelper(options, now) {
  const query = sql`/* mutating */ SELECT 1`
  return mutatingPage(options, 'msg', 'name', 'kind', query, async statement => {
    return read(statement)
  })
}
export async function replacingHelper(options, now) {
  const query = sql`/* replacing */ SELECT 1`
  return replacingPage(options, 'msg', 'name', 'kind', query, async statement => {
    return read(statement)
  })
}
export async function unknownHelper(options, now) {
  const query = sql`/* unknown */ SELECT 1`
  return unknownPage(options, query, async statement => {
    return read(statement)
  })
}
export async function callbackMutates(options, now) {
  const query = sql`/* callbackMutates */ SELECT 1`
  return pageHelper(options, 'msg', 'name', 'kind', query, async statement => {
    statement.strings[0] = 'SELECT 2'
    return read(statement)
  })
}
