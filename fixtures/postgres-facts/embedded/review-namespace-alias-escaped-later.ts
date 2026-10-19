import * as ns from '@example/db'

declare function sendAway(value: unknown): void

// Source order of the two functions must not keep the captured call exact.
export function escaped() {
  const db = ns
  db.query('SELECT id FROM alias_escaped_later')
  const { query } = ns
  query('SELECT id FROM destructure_escaped_later')
}

export function leave() {
  sendAway(ns)
}
