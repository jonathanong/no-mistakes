import sql from 'sql-template-strings'
import { plain, defaults, captured, returned } from './targets.mts'
function localPlain(query, value) { plain(query, value) }
function localDefaults(query, value) { defaults(query, { suffix: 'x' }) }
function localCaptured(query, value) { captured(query, value) }
function localReturned(query, value) { returned(query, value) }
export async function good(value) {
  const query = sql`/* async/plain */ SELECT 1`
  localPlain(query, value)
  return query
}
export async function nonplain(value) {
  const query = sql`/* async/nonplain */ SELECT 1`
  // The inner object satisfies accepts(shape), but not the deferred plain gate.
  localDefaults(query, value)
  return query
}
export async function capturedTarget(value) {
  const query = sql`/* async/captured */ SELECT 1`
  localCaptured(query, value)
  return query
}
export async function returnedTarget(value) {
  const query = sql`/* async/returned */ SELECT 1`
  localReturned(query, value)
  return query
}
