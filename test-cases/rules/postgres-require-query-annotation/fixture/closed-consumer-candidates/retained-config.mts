type RetainedConfig = Readonly<{
  retainedTable: string
  integrityTargetColumn: string
}>

// Protected invariant: freeze both levels and require an own key before reading SQL identifiers.
const configs = Object.freeze({
  relation_user_topic: Object.freeze({
    retainedTable: 'retained_relation_user_topic',
    integrityTargetColumn: 'relation_user_topic_id',
  }),
  relation_post_topic: Object.freeze({
    retainedTable: 'retained_relation_post_topic',
    integrityTargetColumn: 'relation_post_topic_id',
  }),
})

export function getRetainedRelationConfig(name: string): RetainedConfig {
  if (typeof name !== 'string') throw new Error('Invalid relation name')
  if (!Object.hasOwn(configs, name)) throw new Error(`Unknown relation: ${name}`)
  return configs[name as keyof typeof configs]
}

const unsafeConfigs: Record<string, RetainedConfig> = {
  relation_user_topic: {
    retainedTable: 'retained_relation_user_topic',
    integrityTargetColumn: 'relation_user_topic_id',
  },
  relation_post_topic: {
    retainedTable: 'retained_relation_post_topic',
    integrityTargetColumn: 'relation_post_topic_id',
  },
}

export function getUnsafeRetainedRelationConfig(name: string): RetainedConfig {
  // Unsafe control: direct bracket lookup permits inherited properties and mutable values.
  return unsafeConfigs[name]!
}
