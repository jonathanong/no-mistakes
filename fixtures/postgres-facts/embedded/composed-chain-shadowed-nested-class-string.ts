import { query } from "@data-stores/psql";

export function load() {
  class String {
    static raw() {
      return "DELETE FROM users";
    }
  }
  return query(String.raw`SELECT 1`);
}
