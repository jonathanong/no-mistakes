import { read } from 'db'

const OPEN = { github: { emailColumn: process.env.COLUMN } } as const
function getOpen(provider: keyof typeof OPEN) { return OPEN[provider] }
export function openProvider() {
  const config = getOpen('github')
  return read(`/* openProvider */ SELECT ${config.emailColumn} FROM accounts`)
}

export function getRelationTableNameOrThrow() { return process.env.TABLE }
export function wrongModule() {
  const table = getRelationTableNameOrThrow()
  return read(`/* wrongModule */ SELECT * FROM ${table}`)
}
