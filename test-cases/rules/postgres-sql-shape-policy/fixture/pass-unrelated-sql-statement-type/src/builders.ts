import sql, { type SQLStatement as Statement } from 'sql-template-strings'

type StatementLike = { append(fragment: unknown): void }

export function appendUnsafe(query: StatementLike): void {
  query.append(sql`AND EXISTS (
    SELECT 1 FROM topic relation WHERE relation.post_id = posts.id
    UNION ALL
    SELECT 1 FROM topic_alias alias_relation WHERE alias_relation.post_id = posts.id
  )`)
}
