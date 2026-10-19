import * as ns from '@example/db'

declare function sendAway(value: unknown): void

// The escape is later in this function and not in a loop, so both calls stay
// exact. They must still be recorded: dropping either one hides the call.
export function ordered() {
  const db = ns
  db.query('SELECT id FROM alias_ordered')
  const { query } = ns
  query('SELECT id FROM destructure_ordered')
  sendAway(ns)
}
