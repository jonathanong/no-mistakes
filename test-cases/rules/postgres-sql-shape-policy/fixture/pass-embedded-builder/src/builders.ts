import sql, { type SQLStatement } from 'sql-template-strings'
import { RELATION_TABLE } from './schema'

export function buildUncorrelatedTopicFilter(topicId: string): SQLStatement {
  return sql`EXISTS (SELECT 1 FROM `
    .append(RELATION_TABLE)
    .append(sql` relation WHERE relation.topic_id = ${topicId}
      UNION ALL
      SELECT 1 FROM relation__topic_alias alias_relation
      WHERE alias_relation.topic_id = ${topicId})`)
}
