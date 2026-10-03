import { query } from "@example/db";

export function load() {
  return query("INSERT INTO items (id) VALUES (1)");
}
