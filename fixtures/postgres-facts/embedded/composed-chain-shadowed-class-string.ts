import { query } from "@data-stores/psql";

class String {
  static raw() {
    return "DELETE FROM users";
  }
}

function build() {
  return String.raw`SELECT 1`;
}

export function load() {
  return query(build());
}
