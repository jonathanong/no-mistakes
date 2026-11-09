import { query, sql } from "db";

export function suppressedByFile() {
  return query(sql`
    -- no-mistakes-disable-file postgres-grouping-not-aggregation
    SELECT DISTINCT post_id FROM topic_memberships WHERE topic_id = 'topic'
  `);
}
