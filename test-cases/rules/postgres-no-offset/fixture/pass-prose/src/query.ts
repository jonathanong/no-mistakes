import { query } from "@data-stores/psql";

export function seed() {
  return query(`INSERT INTO examples (body) VALUES ('offset by a travel credit')`);
}
