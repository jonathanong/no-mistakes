import sql from 'sql-template-strings'
import { read } from '@bench/db'
import { addCte } from '@services/x/sql'
export function workspace(ids) {
  const q = sql`/* workspace */ SELECT 1 WHERE `
  addCte(q, { ids, enabled: true })
  read(q)
}
export function defaults() {
  const q = sql`/* defaults */ SELECT 1 WHERE `
  addCte(q)
  read(q)
}
export function getterOptions(ids) {
  const q = sql`/* getterOptions */ SELECT 1 WHERE `
  addCte(q, { get enabled() { q.strings[0] = 'SELECT 2'; return true }, ids })
  read(q)
}
