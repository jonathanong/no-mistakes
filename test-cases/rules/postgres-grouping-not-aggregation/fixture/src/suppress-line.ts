import { query, sql } from "db";

export function suppressedByLine(topicId: string) {
  return query(sql`SELECT DISTINCT post_id FROM topic_memberships WHERE topic_id = ${topicId} -- no-mistakes-disable-line postgres-grouping-not-aggregation`);
}
