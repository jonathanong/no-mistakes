import { read } from 'db'

const MUTATED = { github: { emailColumn: 'github_email' } } as const
function getMutated(provider: keyof typeof MUTATED) { return MUTATED[provider] }
export function mutatedProvider() {
  const config = getMutated('github')
  return read(`/* mutatedProvider */ SELECT ${config.emailColumn} FROM accounts`)
}
export function otherCaller() {
  const config = getMutated('github')
  ;(config as { emailColumn: string }).emailColumn = process.env.COLUMN!
}

const ALIASED = { github: { emailColumn: 'github_email' } } as const
function getAliased(provider: keyof typeof ALIASED) { return ALIASED[provider] }
export function aliasedProvider() {
  const config = getAliased('github')
  const alias = config
  ;(alias as { emailColumn: string }).emailColumn = process.env.COLUMN!
  return read(`/* aliasedProvider */ SELECT ${config.emailColumn} FROM accounts`)
}

const RAW = { github: { emailColumn: 'github_email' } } as const
function getRaw(provider: keyof typeof RAW) { return RAW[provider] }
export function rawProvider() {
  const config = getRaw('github')
  Object.assign(getRaw('github'), { emailColumn: process.env.COLUMN })
  return read(`/* rawProvider */ SELECT ${config.emailColumn} FROM accounts`)
}
