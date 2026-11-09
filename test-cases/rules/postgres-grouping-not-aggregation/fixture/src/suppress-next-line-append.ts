import { query, sql } from "db";

export function suppressedByNextLineAppend(topicId: string) {
  // no-mistakes-disable-next-line postgres-grouping-not-aggregation
  const statement = sql`SELECT DISTINCT post_id FROM topic_memberships`;
  statement.append(sql` WHERE topic_id = ${topicId}`);
  return query(statement);
}
