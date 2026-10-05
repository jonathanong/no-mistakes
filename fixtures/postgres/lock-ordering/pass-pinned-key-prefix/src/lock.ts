import { query } from "@example/db";

// Every locked row shares host_id, so ORDER BY guid is (host_id, guid) order.
export function lockByGuid(hostId: string, guids: string[]) {
  return query(`SELECT id, guid FROM feed_items WHERE host_id = $1 AND guid = ANY($2::text[]) ORDER BY guid FOR UPDATE`, [hostId, guids]);
}

// A longer ORDER BY may continue past the remaining key column.
export function lockByGuidThenTieBreak(hostId: string, guids: string[]) {
  return query(`SELECT id, guid FROM feed_items WHERE host_id = $1 AND guid = ANY($2::text[]) ORDER BY guid, something_else FOR UPDATE`, [hostId, guids]);
}

// A qualifier that names the locked relation is still accepted.
export function lockByQualifiedGuid(hostId: string, guids: string[]) {
  return query(`SELECT id, guid FROM feed_items WHERE host_id = $1 AND guid = ANY($2::text[]) ORDER BY feed_items.guid FOR UPDATE`, [hostId, guids]);
}
