import { query } from "@data-stores/psql";

export function load() {
  return query("INSERT INTO items (id) VALUES (1)");
}
