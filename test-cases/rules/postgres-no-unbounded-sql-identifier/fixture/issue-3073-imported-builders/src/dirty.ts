import sql from 'sql-template-strings'
import { type QueryExecutor } from 'db'
import { buildRaw, buildEscaped, rawStatement, mutatedStatement, escapedStatement, consumerMutated, consumerEscaped, consumerAliased } from './unsafe'

consumerMutated.text = process.env.TABLE

export function rawCallback(table: string) {
  const raw = sql``.append(table)
  const select = (execute: QueryExecutor) => execute(`/* raw callback */ SELECT ${raw.text}`, raw.values)
  return select
}
export function rawFactory(table: string) {
  const raw = buildRaw(table)
  const select = (execute: QueryExecutor) => execute(`/* raw factory */ SELECT ${raw.text}`, raw.values)
  return select
}
export function escapedFactory() {
  const escaped = buildEscaped()
  const select = (execute: QueryExecutor) => execute(`/* escaped factory */ SELECT ${escaped.text}`, escaped.values)
  return select
}
export const importedRaw = (execute: QueryExecutor) => execute(`/* imported raw */ SELECT ${rawStatement.text}`, rawStatement.values)
export const importedMutation = (execute: QueryExecutor) => execute(`/* producer write */ SELECT ${mutatedStatement.text}`, mutatedStatement.values)
export const importedEscape = (execute: QueryExecutor) => execute(`/* producer escape */ SELECT ${escapedStatement.text}`, escapedStatement.values)
export const consumerMutation = (execute: QueryExecutor) => execute(`/* consumer write */ SELECT ${consumerMutated.text}`, consumerMutated.values)

declare function escape(value: unknown): void
escape(consumerEscaped)
const alias = consumerAliased
alias.text = process.env.TABLE

export const consumerEscape = (execute: QueryExecutor) => execute(`/* consumer escape */ SELECT ${consumerEscaped.text}`, consumerEscaped.values)
export const consumerAlias = (execute: QueryExecutor) => execute(`/* consumer alias */ SELECT ${consumerAliased.text}`, consumerAliased.values)

import { buildCycleA } from './cycle-a'
export function cyclicFactory() {
  const cycleStatement = buildCycleA()
  const select = (execute: QueryExecutor) => execute(`/* cycle */ SELECT ${cycleStatement.text}`, cycleStatement.values)
  return select
}
