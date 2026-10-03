import { query } from "@example/db";

export function seed() {
  return query(`INSERT INTO examples (body) VALUES ('offset by a travel credit')`);
}
