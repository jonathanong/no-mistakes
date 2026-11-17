import SQL from "sql-template-strings";
import { query } from "@example/db";
import { HARD_MAX } from "./limits";

export function add(conversationId: string, userIds: string[], roles: string[]) {
  // no-mistakes-disable-next-line postgres-conflict-ordering
  return query(SQL`WITH requested AS (SELECT u.user_id, u.role FROM unnest(${userIds}::uuid[], ${roles}::text[]) AS u(user_id, role)), existing AS (SELECT p.user_id FROM participants p WHERE p.conversation_id = ${conversationId} ORDER BY p.user_id LIMIT ${HARD_MAX}) INSERT INTO participants (conversation_id, user_id, role) SELECT ${conversationId}, r.user_id, r.role FROM requested r ON CONFLICT (conversation_id, user_id) DO NOTHING`);
}

export function addUnsuppressed(conversationId: string, userIds: string[], roles: string[]) {
  return query(SQL`WITH requested AS (SELECT u.user_id, u.role FROM unnest(${userIds}::uuid[], ${roles}::text[]) AS u(user_id, role)), existing AS (SELECT p.user_id FROM participants p WHERE p.conversation_id = ${conversationId} ORDER BY p.user_id LIMIT ${HARD_MAX}) INSERT INTO participants (conversation_id, user_id, role) SELECT ${conversationId}, r.user_id, r.role FROM requested r ON CONFLICT (conversation_id, user_id) DO NOTHING`);
}
