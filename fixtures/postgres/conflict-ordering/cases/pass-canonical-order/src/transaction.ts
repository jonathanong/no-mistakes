import { withTransaction } from "@data-stores/psql";

export function insert(ids: string[]) {
  return withTransaction(async () => query(`
    INSERT INTO items (id)
    SELECT input.id FROM unnest($1::uuid[]) AS input(id)
    ORDER BY input.id
    ON CONFLICT (id) DO NOTHING
  `, [ids]));
}
