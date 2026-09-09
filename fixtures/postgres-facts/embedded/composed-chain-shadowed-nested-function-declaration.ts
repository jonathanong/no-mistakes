import { query } from "@data-stores/psql";

function build() {
  return "DELETE FROM users";
}

export function run() {
  function build() {
    return "SELECT * FROM topics";
  }
  return query(build());
}
