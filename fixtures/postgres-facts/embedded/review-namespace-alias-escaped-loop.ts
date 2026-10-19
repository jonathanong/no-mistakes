import * as ns from '@example/db'

declare function sendAway(value: unknown): void

// A later loop escape reaches this captured call even though the write is after it.
export function looped() {
  const { query } = ns
  query('SELECT id FROM destructure_escaped_loop')
  for (let i = 0; i < 1; i += 1) {
    sendAway(ns)
  }
}
