import SQL from "sql-template-strings";
import { query } from "@example/db";
import { HARD_MAX } from "./limits";

export function addWithCapacity(cid: string, userIds: string[], roles: string[]) {
  // no-mistakes-disable-next-line postgres-conflict-ordering
  return query(SQL`WITH input AS (SELECT * FROM unnest(${userIds}::uuid[], ${roles}::text[]) AS incoming(user_id, role)), census AS (SELECT count(*)::int AS total FROM (SELECT 1 FROM participants WHERE conversation_id = ${cid} LIMIT ${HARD_MAX}) capped), free_slots AS (SELECT slot FROM generate_series(1, ${HARD_MAX}) AS slot), inserted AS (INSERT INTO participants (conversation_id, user_id, role) SELECT ${cid}, i.user_id, i.role FROM input i ON CONFLICT DO NOTHING RETURNING user_id) SELECT count(*) FROM inserted`);
}

export function addWithCapacityUnsuppressed(cid: string, userIds: string[], roles: string[]) {
  return query(SQL`WITH input AS (SELECT * FROM unnest(${userIds}::uuid[], ${roles}::text[]) AS incoming(user_id, role)), census AS (SELECT count(*)::int AS total FROM (SELECT 1 FROM participants WHERE conversation_id = ${cid} LIMIT ${HARD_MAX}) capped), free_slots AS (SELECT slot FROM generate_series(1, ${HARD_MAX}) AS slot), inserted AS (INSERT INTO participants (conversation_id, user_id, role) SELECT ${cid}, i.user_id, i.role FROM input i ON CONFLICT DO NOTHING RETURNING user_id) SELECT count(*) FROM inserted`);
}
