import sql from 'sql-template-strings'
import { read } from '@bench/db'

export function mutable(flag, cursor) {
  let statement = sql`/* mutable initial */ SELECT 1`.append(cursor)
  if (flag) statement = sql`/* mutable alternate */ SELECT 2`.append(cursor)
  statement.append(sql` ORDER BY id`).append(cursor)
  read(statement) // low: true
}

export function missingAssignment(flag, cursor) {
  let statement = sql`/* mutable initial */ SELECT 1`.append(cursor)
  if (flag) statement = sql`SELECT 2`.append(cursor)
  statement.append(sql` ORDER BY id`)
  read(statement) // low: false
}

export function nonliteralAssignment(flag, cursor, dynamic) {
  let statement = sql`/* mutable initial */ SELECT 1`.append(cursor)
  if (flag) statement = dynamic
  statement.append(sql` ORDER BY id`)
  read(statement) // low: false
}

export function missingInitial(flag, cursor) {
  let statement = sql`SELECT 1`.append(cursor)
  if (flag) statement = sql`/* alternate only */ SELECT 2`.append(cursor)
  read(statement) // low: false
}

export function storedAlias(flag, cursor, store) {
  let statement = sql`/* stored initial */ SELECT 1`.append(cursor)
  if (flag) statement = sql`/* stored alternate */ SELECT 2`.append(cursor)
  store.query = statement
  read(statement) // low: false
}

export function escapedAssignment(flag, cursor, opaque) {
  let statement = sql`/* escaped initial */ SELECT 1`.append(cursor)
  if (flag) opaque(statement = sql`/* escaped alternate */ SELECT 2`.append(cursor))
  read(statement) // low: false
}

export function rewrittenHead(flag, cursor) {
  let statement = sql`/* rewritten initial */ SELECT 1`.append(cursor)
  if (flag) statement = sql`/* rewritten alternate */ SELECT 2`.append(cursor)
  statement.strings[0] = 'SELECT 3'
  read(statement) // low: false
}

export function capturedWrite(flag, cursor) {
  let statement = sql`/* captured initial */ SELECT 1`.append(cursor)
  function replace() { statement = sql`/* captured alternate */ SELECT 2`.append(cursor) }
  replace()
  read(statement) // low: false
}
