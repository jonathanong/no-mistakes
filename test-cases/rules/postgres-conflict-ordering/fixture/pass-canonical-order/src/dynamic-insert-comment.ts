import { query } from "@data-stores/psql";

export function select(suffix: string) {
  return query(`SELECT 1 /* INSERT is documentation */ ${suffix}`);
}
