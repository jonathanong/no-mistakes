import { read } from 'db'

const ESCAPED = { github: { emailColumn: 'github_email' } } as const
function getEscaped(provider: keyof typeof ESCAPED) { return ESCAPED[provider] }
declare function consume(value: unknown): void
consume(getEscaped)
export function escapedProvider() {
  const config = getEscaped('github')
  return read(`/* escapedProvider */ SELECT ${config.emailColumn} FROM accounts`)
}

export const EXPORTED = { github: { emailColumn: 'github_email' } } as const
function getExported(provider: keyof typeof EXPORTED) { return EXPORTED[provider] }
export function exportedProvider() {
  const config = getExported('github')
  return read(`/* exportedProvider */ SELECT ${config.emailColumn} FROM accounts`)
}

const GETTER = { github: { get emailColumn() { return process.env.COLUMN } } }
function getGetter(provider: keyof typeof GETTER) { return GETTER[provider] }
export function getterProvider() {
  const config = getGetter('github')
  return read(`/* getterProvider */ SELECT ${config.emailColumn} FROM accounts`)
}
