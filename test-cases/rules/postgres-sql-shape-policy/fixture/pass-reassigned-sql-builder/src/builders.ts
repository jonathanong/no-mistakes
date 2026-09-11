import sql from 'sql-template-strings'

export function appendUnsafe(): void {
  let query = sql``
  query = sql``
  query.append(sql`AND EXISTS (
    SELECT 1 FROM topic relation WHERE relation.post_id = posts.id
    UNION ALL
    SELECT 1 FROM topic_alias alias_relation WHERE alias_relation.post_id = posts.id
  )`)
}
