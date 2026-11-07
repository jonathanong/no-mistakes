import { read } from 'db'
import * as providers from './namespace-member-provider.ts'
import { getMemberIdentifiers } from './namespace-member-provider.ts'
declare function consume(value: unknown): void
consume(providers.getMemberIdentifiers)
export function namespaceMemberEscape() {
  const config = getMemberIdentifiers('github')
  return read(`/* namespaceMemberEscape */ SELECT ${config.emailColumn} FROM accounts`)
}
