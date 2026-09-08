import { query } from "@data-stores/psql";

export function insert(externalIds: string[]) {
  return query(`
    INSERT INTO items (external_id)
    SELECT input.external_id FROM unnest($1::text[]) AS input(external_id)
    ORDER BY input.external_id
    ON CONFLICT ON CONSTRAINT items_external_id_key DO NOTHING
  `, [externalIds]);
}
