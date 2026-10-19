import * as ns from '@example/db'

declare function sendAway(value: unknown): void

// This escape is visited first. Both later calls are in another function, so
// they are unanalyzable. The destructured `query` must not disappear.
export function leave() {
  sendAway(ns)
}

export function escaped() {
  const db = ns
  db.query('SELECT id FROM alias_escaped')
  const { query } = ns
  query('SELECT id FROM destructure_escaped')
}
