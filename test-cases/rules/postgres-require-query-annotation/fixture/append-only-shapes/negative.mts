import sql from 'sql-template-strings'
import { read } from '@bench/db'
let stored
function mutate(q) { q.strings[0] = 'SELECT 2' }
function reassign(q) { q = sql`SELECT 2`; q.append(' WHERE true') }
function store(q) { stored = q; q.append(' WHERE true') }
export function mutation() {
  const q = sql`/* mutation */ SELECT 1`
  mutate(q)
  read(q)
}
export function reassignment() {
  const q = sql`/* reassignment */ SELECT 1`
  reassign(q)
  read(q)
}
export function storage() {
  const q = sql`/* storage */ SELECT 1`
  store(q)
  read(q)
}
export function opaqueCall() {
  const q = sql`/* opaque */ SELECT 1`
  opaque(q)
  read(q)
}
export function dynamic(head) {
  const q = sql`${head} SELECT 1`
  addUnknown(q)
  read(q)
}
