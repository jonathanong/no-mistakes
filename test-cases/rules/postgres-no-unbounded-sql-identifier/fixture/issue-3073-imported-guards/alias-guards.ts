import { write } from 'db'
import { PRODUCER_ALIAS_TABLES } from './alias-producer.ts'
import { CONSUMER_ALIAS_TABLES, OTHER_ALIAS_TABLES, NAMESPACE_ALIAS_TABLES, EXPORTED_ALIAS_TABLES } from './alias-tables.ts'
import { NAMESPACE_OBJECT_TABLES } from './alias-namespace-object-tables.ts'
import { SHARED_TABLES } from './alias-ambiguous-a.ts'

export function producerAlias(table: string) {
  if (!PRODUCER_ALIAS_TABLES.includes(table)) throw new Error('bad')
  return write(`/* producerAlias */ SELECT * FROM ${table}`)
}
export function consumerAlias(table: string) {
  const alias = CONSUMER_ALIAS_TABLES as unknown as string[]
  alias.push(table)
  if (!CONSUMER_ALIAS_TABLES.includes(table)) throw new Error('bad')
  return write(`/* consumerAlias */ SELECT * FROM ${table}`)
}
export function otherImporterAlias(table: string) {
  if (!OTHER_ALIAS_TABLES.includes(table)) throw new Error('bad')
  return write(`/* otherImporterAlias */ SELECT * FROM ${table}`)
}
export function namespaceAlias(table: string) {
  if (!NAMESPACE_ALIAS_TABLES.includes(table)) throw new Error('bad')
  return write(`/* namespaceAlias */ SELECT * FROM ${table}`)
}
export function namespaceObjectAlias(table: string) {
  if (!NAMESPACE_OBJECT_TABLES.includes(table)) throw new Error('bad')
  return write(`/* namespaceObjectAlias */ SELECT * FROM ${table}`)
}
export function ambiguousAlias(table: string) {
  if (!SHARED_TABLES.includes(table)) throw new Error('bad')
  return write(`/* ambiguousAlias */ SELECT * FROM ${table}`)
}
export function exportedAlias(table: string) {
  if (!EXPORTED_ALIAS_TABLES.includes(table)) throw new Error('bad')
  return write(`/* exportedAlias */ SELECT * FROM ${table}`)
}
