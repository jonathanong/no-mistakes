import sql from 'sql-template-strings'
import { read } from '@bench/db'
function fresh(q) { q.append(' WHERE true'); return sql`SELECT 2` }
function ignored(q) { return sql`SELECT 2` }
function early(q, flag) { if (flag) return; return q.append(' WHERE true') }
function alternate(q, flag) { if (flag) return sql`SELECT 2`; return q.append(' WHERE true') }
function returning(q) { return q.append(' WHERE true') }
function selfSuffix(q) { q.append(opaque(q)) }
export function freshReturn() {
  const q = sql`/* fresh */ SELECT 1`
  fresh(q).append(' LIMIT 1')
  read(q)
}
export function ignoredInput() {
  const q = sql`/* ignored */ SELECT 1`
  ignored(q).append(' LIMIT 1')
  read(q)
}
export function earlyReturn(flag) {
  const q = sql`/* early */ SELECT 1`
  early(q, flag).append(' LIMIT 1')
  read(q)
}
export function alternateReturn(flag) {
  const q = sql`/* alternate */ SELECT 1`
  alternate(q, flag).append(' LIMIT 1')
  read(q)
}
export function storedAlias() {
  const q = sql`/* stored */ SELECT 1`
  const alias = returning(q)
  alias.append(' LIMIT 1')
  read(q)
}
export function nestedEscape() {
  const q = sql`/* nested */ SELECT 1`
  selfSuffix(q)
  read(q)
}
