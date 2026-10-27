import { query } from "@example/db";

export function broken() {
  return query(`INSERT INTO items (id) VALUES ( ON CONFLICT (id) DO NOTHING`);
}
