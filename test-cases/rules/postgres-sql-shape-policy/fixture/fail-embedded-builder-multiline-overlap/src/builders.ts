import sql from 'sql-template-strings'

export function build(topicId: string): unknown {
  const query = sql`AND true `
  return query
    .append(sql`EXISTS (
      SELECT 1 FROM topic relation WHERE relation.post_id = ${topicId}
      UNION ALL
      SELECT 1 FROM topic_alias alias_relation WHERE alias_relation.post_id = posts.id
    )`)
}
