import { query } from "@example/db";

// host_id is not pinned, and the partial guid index must not make this order valid.
export function unpinnedLeading(guids: string[]) {
  return query(`SELECT id, guid FROM feed_items WHERE guid = ANY($1::text[]) ORDER BY guid FOR UPDATE`, [guids]);
}

// An equality inside OR does not pin host_id.
export function orWrappedPin(hostId: string, guids: string[]) {
  return query(`SELECT id, guid FROM feed_items WHERE (host_id = $1 OR host_id = $1) AND guid = ANY($2::text[]) ORDER BY guid FOR UPDATE`, [hostId, guids]);
}

// other_col is not the remaining unique-key column.
export function wrongColumn(hostId: string, guids: string[]) {
  return query(`SELECT id, guid FROM feed_items WHERE host_id = $1 AND guid = ANY($2::text[]) ORDER BY other_col FOR UPDATE`, [hostId, guids]);
}

// The catalog key is ASC NULLS LAST; DESC is a different lock order.
export function wrongDirection(hostId: string, guids: string[]) {
  return query(`SELECT id, guid FROM feed_items WHERE host_id = $1 AND guid = ANY($2::text[]) ORDER BY guid DESC FOR UPDATE`, [hostId, guids]);
}

// NULLS FIRST does not match the catalog key's nulls ordering.
export function wrongNulls(hostId: string, guids: string[]) {
  return query(`SELECT id, guid FROM feed_items WHERE host_id = $1 AND guid = ANY($2::text[]) ORDER BY guid NULLS FIRST FOR UPDATE`, [hostId, guids]);
}

// Pinning guid, the second key column, does not let ORDER BY skip host_id.
export function nonLeadingPin(guid: string, hostIds: string[]) {
  return query(`SELECT id, guid FROM feed_items WHERE guid = $1 AND host_id = ANY($2::text[]) ORDER BY host_id FOR UPDATE`, [guid, hostIds]);
}

// A pin on another relation does not fix feed_items.host_id.
export function otherRelation(hostId: string, guids: string[]) {
  return query(`SELECT f.id FROM feed_items f JOIN hosts h ON true WHERE h.id = $1 AND f.guid = ANY($2::text[]) ORDER BY guid FOR UPDATE OF f`, [hostId, guids]);
}
