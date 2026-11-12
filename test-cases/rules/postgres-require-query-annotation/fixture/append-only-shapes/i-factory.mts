import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { appendTypeFilter } from './i-type-filter.mts'
import { importedFactory } from './i-factory-lib.mts'
const shared = sql`/* shared */ SELECT 1`
function columns(alias) { return sql`, ${alias}.name` }
function buildSelect() { return sql` FROM c` }
function startCandidates(user) {
  return sql`/* searchCommunities */ WITH cands AS (SELECT 1)`
}
function maybeStart(flag) {
  if (flag) return sql`/* maybe */ SELECT 1`
  return sql`SELECT 2`
}
function passthrough(q) { return q }
function sharedFactory() { return shared }
function twoLiteral(flag) {
  if (flag) return sql`/* one */ SELECT 1`
  return sql`/* two */ SELECT 2`
}
export async function communities(user, type) {
  const q = startCandidates(user)
  appendTypeFilter(q, user, type)
  q.append(columns('c'))
  q.append(buildSelect())
  await read(q)
}
export async function imported(user, type) {
  const q = importedFactory(user)
  appendTypeFilter(q, user, type)
  await read(q)
}
export async function twoLiterals(flag) {
  const q = twoLiteral(flag)
  await read(q)
}
export async function nonLiteralReturn(flag) {
  const q = maybeStart(flag)
  await read(q)
}
export async function returnsParameter() {
  const q = passthrough(sql`/* param */ SELECT 1`)
  await read(q)
}
export async function returnsShared() {
  const q = sharedFactory()
  await read(q)
}
export async function factoryMutated(user) {
  const q = startCandidates(user)
  q.strings[0] = 'SELECT 2'
  await read(q)
}
