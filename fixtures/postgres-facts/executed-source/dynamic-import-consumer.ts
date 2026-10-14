import { read } from '@example/db'
import { query } from './eager-builder'

export async function list() {
  await import('./eager-mutator')
  return read(query)
}
