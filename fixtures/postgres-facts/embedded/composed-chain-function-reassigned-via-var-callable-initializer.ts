import { query } from "@data-stores/psql";

function build() {
  return "SELECT id FROM topics";
}

var build = () => "DELETE FROM users";

const sql = build();

export function load() {
  return query(sql);
}
