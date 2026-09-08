import { query } from "@data-stores/psql";

export function insertItems() {
  return query(`INSERT INTO items (id) SELECT input.id AS conflict_id FROM input ORDER BY conflict_id ON CONFLICT (id) DO NOTHING`);
}
