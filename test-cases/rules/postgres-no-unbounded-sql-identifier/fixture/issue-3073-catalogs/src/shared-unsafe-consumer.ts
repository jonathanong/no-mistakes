import { read } from 'db'
import { getShared, getSharedAgain } from './shared-unsafe-barrel.ts'

declare function consume(value: unknown): void
consume(getShared)
export function importedEscape() {
  const config = getShared('github')
  return read(`/* importedEscape */ SELECT ${config.emailColumn} FROM accounts`)
}
export function sharedSource() {
  const config = getSharedAgain('github')
  return read(`/* sharedSource */ SELECT ${config.emailColumn} FROM accounts`)
}
