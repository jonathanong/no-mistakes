const RELATIONS = [
  {
    subject: 'topic', predicate: 'parent', object: 'topic',
    table_name: 'rel_topic_parent_topic',
  },
] as const

export function getRelationOrThrow(l: {
  subject: string; predicate: string; object: string
}) {
  const r = RELATIONS.find(
    x => x.subject === l.subject &&
      x.predicate === l.predicate && x.object === l.object,
  )
  if (r) return r
  throw new Error('missing relation')
}

export function getRelationTableNameOrThrow(
  l: Parameters<typeof getRelationOrThrow>[0],
): string {
  return getRelationOrThrow(l).table_name
}
