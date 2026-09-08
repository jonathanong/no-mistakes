import { query } from "@data-stores/psql";

export function insert(ids: string[]) {
  return query(`
    INSERT INTO items (id)
    SELECT input.id FROM unnest($1::uuid[]) AS input(id)
    ORDER BY input.id
    ON CONFLICT DO NOTHING
  `, [ids]);
}
