import { read } from '@bench/db'
import { getRetainedRelationConfig, getUnsafeRetainedRelationConfig } from './retained-config.mts'

type RootFamily =
  | 'user'
  | 'post'
  | 'topic'
  | 'url'
  | 'rss-item'
  | 'oauth-client'
  | 'api-key'

function assertNever(value: never): never {
  throw new Error(`Unknown root family: ${value}`)
}

function retainedRootTable(family: RootFamily): string {
  switch (family) {
    case 'user': return 'retained_users'
    case 'post': return 'retained_posts'
    case 'topic': return 'retained_topics'
    case 'url': return 'retained_urls'
    case 'rss-item': return 'retained_rss_items'
    case 'oauth-client': return 'retained_oauth_clients'
    case 'api-key': return 'retained_api_keys'
    default: return assertNever(family)
  }
}

function retainedRootTableWithUnknownDefault(family: string): string | undefined {
  switch (family) {
    case 'user': return 'retained_users'
    case 'post': return 'retained_posts'
    default: return undefined
  }
}

export function readRetainedRelation(table: string): void {
  const config = getRetainedRelationConfig(table)
  read(`/* insertRetainedOwner */ INSERT INTO ${config.retainedTable} (subject_id, id) VALUES ($1, $2)`, ['subject', 'id'])
  read(`/* insertRetainedImpact */ INSERT INTO relation_impacts (${config.integrityTargetColumn}) VALUES ($1)`, ['id'])
  read(
    `/* readRetainedRelation */ SELECT id FROM ${config.retainedTable} WHERE ${config.integrityTargetColumn} = $1`,
    ['id'],
  )
  read(`/* countRetainedRelation */ SELECT COUNT(*) FROM ${config.retainedTable}`)
}

export function readRetainedRoot(family: RootFamily): void {
  const table = retainedRootTable(family)
  read(`/* readRetainedRoot */ SELECT id FROM ${table} WHERE id = $1`, ['id'])
}

export function readUnknownDefaultControl(family: string): void {
  const table = retainedRootTableWithUnknownDefault(family)
  // Unsafe control: an unknown selector does not throw and can leave the table unresolved.
  read(`/* readUnknownDefaultControl */ SELECT id FROM ${table} WHERE id = $1`, ['id'])
}

export function readUnsafeRetainedRelation(table: string): void {
  const config = getUnsafeRetainedRelationConfig(table)
  // Unsafe control: writable registry lookup is not a closed SQL identifier proof.
  read(`/* readUnsafeRetainedRelation */ SELECT id FROM ${config.retainedTable} WHERE id = $1`, ['id'])
}
