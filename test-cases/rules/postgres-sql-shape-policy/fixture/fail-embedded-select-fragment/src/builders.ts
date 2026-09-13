import sql, { type SQLStatement } from 'sql-template-strings'

export function buildUnsafeSelect(): SQLStatement {
  return sql`SELECT 1 FROM posts WHERE EXISTS (
    SELECT 1 FROM relation__topic relation
    WHERE relation.subject_id = posts.id
    UNION ALL
    SELECT 1 FROM relation__topic_alias alias_relation
    WHERE alias_relation.subject_id = posts.id
  )`
}
