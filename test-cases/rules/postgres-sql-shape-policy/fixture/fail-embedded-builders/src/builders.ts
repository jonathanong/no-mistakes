import sql, { type SQLStatement } from 'sql-template-strings'
import { RELATION_TABLE } from './schema'

export function buildTopicMembershipExists(postIdColumn: string, topicId: string): SQLStatement {
  return sql`EXISTS (SELECT 1 FROM `
    .append(RELATION_TABLE)
    .append(sql` relation WHERE relation.subject_id = `)
    .append(postIdColumn)
    .append(sql` UNION ALL SELECT 1 FROM relation__post__topic_alias alias_relation
      WHERE alias_relation.subject_id = ${topicId})`)
}

export function appendHashtagFilters(query: SQLStatement, topicIds: string[]): void {
  for (const topicId of topicIds) {
    query.append(sql`AND EXISTS (
      SELECT 1 FROM relation__rss_feed_item__topic relation
      WHERE relation.subject_id = rss_feed_items.id AND relation.object_id = ${topicId}
      UNION ALL
      SELECT 1 FROM relation__rss_feed_item__topic_alias alias_relation
      WHERE alias_relation.subject_id = rss_feed_items.id AND alias_relation.topic_id = ${topicId}
    )`)
  }
}

export function buildHashtagTopicFilter(topicId: string): SQLStatement {
  return sql`EXISTS (
    SELECT 1 FROM relation__rss_feed_item__topic relation
    WHERE relation.subject_id = rss_feed_items.id AND relation.object_id = ${topicId}
    UNION ALL
    SELECT 1 FROM relation__rss_feed_item__topic_alias alias_relation
    WHERE alias_relation.subject_id = rss_feed_items.id AND alias_relation.topic_id = ${topicId}
  )`
}
