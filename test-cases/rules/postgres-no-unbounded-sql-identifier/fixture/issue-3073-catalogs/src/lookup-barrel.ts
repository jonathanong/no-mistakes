import { read } from 'db'
import { providerIdentifiers as identifiers } from './provider-barrel.ts'

export function barrelLookup(provider: 'github' | 'gitlab') {
  const config = identifiers(provider)
  const email = config.emailColumn
  return read(`/* barrelLookup */ SELECT ${email} FROM accounts`)
}
