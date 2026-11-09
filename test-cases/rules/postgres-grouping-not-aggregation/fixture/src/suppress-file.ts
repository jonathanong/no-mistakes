// no-mistakes-disable-file postgres-grouping-not-aggregation
import { query, sql } from "db";

export function suppressedByFile() {
  return query(sql`
    SELECT DISTINCT post_id FROM topic_memberships WHERE topic_id = '00000000-0000-0000-0000-000000000001'
  `);
}
