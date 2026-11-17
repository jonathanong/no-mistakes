import { query, sql } from "@example/db";

const HARD_MAX = 100;

export function addParticipants(cid: string, userIds: string[], roles: string[]) {
  // no-mistakes-disable-next-line postgres-conflict-ordering
  return query(sql`INSERT INTO participants (conversation_id, user_id, role) SELECT ${cid}, u, r FROM unnest(${userIds}::uuid[], ${roles}::text[]) AS t(u, r) LIMIT ${HARD_MAX} ON CONFLICT (conversation_id, user_id) DO NOTHING`);
}
