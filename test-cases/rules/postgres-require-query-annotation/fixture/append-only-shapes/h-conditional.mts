import sql from 'sql-template-strings'
import { read } from '@bench/db'

export function conditional(flag, cursor, suffix) {
  const q = flag
    ? sql`/* conditional first */ SELECT 1`
    : sql`/* conditional second */ SELECT 2`.append(cursor).append(suffix)
  read(q) // low: true
}

export function conditionalMissing(flag, cursor) {
  const q = flag ? sql`/* first only */ SELECT 1` : sql`SELECT 2`.append(cursor)
  read(q) // low: false
}

export function conditionalDynamic(flag, cursor, dynamic) {
  const q = flag ? sql`/* first only */ SELECT 1` : dynamic.append(cursor)
  read(q) // low: false
}

export function conditionalOpaque(flag, cursor, opaque) {
  const q = flag ? sql`/* first */ SELECT 1` : sql`/* second */ SELECT 2`.append(cursor)
  opaque(q)
  read(q) // low: false
}

export function conditionalStored(flag, cursor, store) {
  const q = flag ? sql`/* first */ SELECT 1` : sql`/* second */ SELECT 2`.append(cursor)
  store.query = q
  read(q) // low: false
}

export function conditionalRewritten(flag, cursor) {
  const q = flag ? sql`/* first */ SELECT 1` : sql`/* second */ SELECT 2`.append(cursor)
  q.strings[0] = 'SELECT 3'
  read(q) // low: false
}

export function conditionalNonliteralTag(flag, cursor, other) {
  const q = flag ? sql`/* first */ SELECT 1` : other`/* second */ SELECT 2`.append(cursor)
  read(q) // low: false
}

export function conditionalUsedAppend(flag, cursor) {
  const q = flag ? sql`/* first */ SELECT 1` : sql`/* second */ SELECT 2`.append(cursor)
  const alias = q.append(sql` ORDER BY id`)
  alias.strings[0] = 'SELECT 3'
  read(q) // low: false
}
