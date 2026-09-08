import { query } from "@data-stores/psql";

export function select(ids: string[], suffix: string) {
  return query(`SELECT id FROM items WHERE id = ANY($1) ${suffix}`, [ids]);
}
