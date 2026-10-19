import { pool, replacement } from '@example/db'

declare function withTx(pool: unknown, run: () => void): void

// Passing an imported binding does not rebind it. A `.query` write does.
export function kept() {
  return pool.query('SELECT id FROM kept')
}

export function passPool() {
  withTx(pool, () => undefined)
}

export function lost() {
  return replacement.query('SELECT id FROM lost')
}

export function writeReplacement() {
  replacement.query = () => undefined
}
