import { pool, replacement } from '@example/db'
import { withTx } from './tx'
import legacy from './legacy'
import * as helpers from './helpers'
import type { Tx } from './types'
import { type OnlyType, real } from './types'
import './side-effect'

const sink = function (value: unknown) {
  return value
}

declare function ambient(value: unknown): void

// Passing an imported binding to an imported helper does not rebind it.
export function kept() {
  return pool.query('SELECT id FROM kept')
}

export function passPool() {
  withTx(pool, () => undefined)
  legacy(pool)
  real(pool)
  ambient(pool)
}

export function lost() {
  return replacement.query('SELECT id FROM lost')
}

export function writeReplacement() {
  replacement.query = () => undefined
}
