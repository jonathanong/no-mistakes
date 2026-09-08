import { query } from "@data-stores/psql";

export function insert(ids: string[]) {
  return query(`
    INSERT INTO items (id)
    SELECT input.id AS ordered_id
    FROM unnest($1::uuid[]) AS input(id)
    ORDER BY ordered_id
    ON CONFLICT (id) DO NOTHING
  `, [ids]);
}
