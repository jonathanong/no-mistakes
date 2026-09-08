import { query } from "@data-stores/psql";

export function select(suffix: string) {
  return query(`SELECT 'INSERT is documentation' AS note ${suffix}`);
}
