import { query } from "@example/db";

export function insert(ids: string[]) {
  return query(`
    INSERT INTO items (id)
    SELECT input.id
    FROM unnest($1::uuid[]) AS input(id)
    ON CONFLICT (id) DO NOTHING
  `, [ids]);
}
