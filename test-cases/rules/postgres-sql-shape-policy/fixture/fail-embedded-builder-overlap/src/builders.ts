import sql from 'sql-template-strings'

export function build(): unknown {
  const query = sql``
  return query.append(sql`EXISTS (
    SELECT 1 FROM topic relation WHERE relation.post_id = posts.id
    UNION ALL
    SELECT 1 FROM topic_alias alias_relation WHERE alias_relation.post_id = posts.id
  )`)
}
