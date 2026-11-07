import { read } from 'db'
import * as providers from './namespace-object-provider.ts'
import { getObjectIdentifiers } from './namespace-object-provider.ts'
declare function consume(value: unknown): void
consume(providers)
export function namespaceObjectEscape() {
  const config = getObjectIdentifiers('github')
  return read(`/* namespaceObjectEscape */ SELECT ${config.emailColumn} FROM accounts`)
}
