import { query } from "@example/db";
export function run(flag: boolean, opaque: string) {
  query("SELECT 1");
  query(opaque);
  query(flag
    ? "SELECT id FROM accounts ORDER BY id FOR UPDATE"
    : "SELECT id FROM accounts FOR UPDATE");
  query(flag
    ? "INSERT INTO accounts (id) VALUES (1) ON CONFLICT (id) DO NOTHING"
    : "INSERT INTO accounts (id) VALUES (2) ON CONFLICT (id) DO NOTHING");
  query(flag
    ? "/* @query first */ SELECT 1"
    : "/* @query second */ SELECT 2");
}
