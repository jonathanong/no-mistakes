import { query } from "@data-stores/psql";

export function insert(ids: string[], suffix: string) {
  let sql = `INSERT INTO items (id) SELECT input.id FROM unnest($1::uuid[]) AS input(id) ${suffix}`;
  return query(sql, [ids]);
}
