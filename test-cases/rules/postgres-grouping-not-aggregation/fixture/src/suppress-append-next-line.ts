import { query, sql } from "@example/db";

export function suppressedInAppendedFragment(topicId: string) {
  const statement = sql`SELECT DISTINCT post_id FROM topic_memberships`;
  statement.append(sql`
    -- no-mistakes-disable-next-line postgres-grouping-not-aggregation
    WHERE topic_id = ${topicId}
  `);
  return query(statement);
}
