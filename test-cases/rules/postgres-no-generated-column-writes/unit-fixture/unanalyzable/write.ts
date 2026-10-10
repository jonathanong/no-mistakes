import { write } from '@example/db'

// Opaque executor argument: no SQL text is recovered (unanalyzable, line 5).
export function opaque(sql: string) {
  return write(sql)
}

// Dynamic UPDATE of a table with a generated column (unanalyzable, line 10).
export function touchItem(column: string) {
  return write(`UPDATE items SET ${column} = now()`)
}

// Dynamic INSERT into an interpolated table could target `items` (unanalyzable, line 15).
export function insertInto(table: string) {
  return write(`INSERT INTO ${table} (note) VALUES ($1)`, ['x'])
}

// An opaque tail can append a second write to a tracked table; fail closed.
export function touchLog(column: string) {
  return write(`UPDATE logs SET ${column} = now()`)
}

// Dynamic SELECT and DELETE cannot assign columns, so they stay quiet.
export function read(column: string) {
  return write(`SELECT id FROM items ORDER BY ${column}`)
}

export function remove(table: string) {
  return write(`DELETE FROM ${table} WHERE id = $1`, ['x'])
}

// Line-level suppression applies to unanalyzable findings like any other finding.
export function suppressed(sql: string) {
  // no-mistakes-disable-next-line postgres-no-generated-column-writes
  return write(sql)
}

export function cteWrite(column: string) {
  return write(`WITH changed AS (UPDATE items SET ${column} = now() RETURNING *) SELECT * FROM changed`)
}
export function cteDelete(column: string) {
  return write(`WITH changed AS (UPDATE items SET ${column} = now() RETURNING *) DELETE FROM logs`)
}
