import { query } from '@data-stores/psql'
import sql from 'sql-template-strings'

function buildUnsafeTopicFilter() {
  return sql`EXISTS (
    SELECT 1 FROM relation__topic relation
    WHERE relation.subject_id = posts.id
    UNION ALL
    SELECT 1 FROM relation__topic_alias alias_relation
    WHERE alias_relation.subject_id = posts.id
  )`
}

query(buildUnsafeTopicFilter())
