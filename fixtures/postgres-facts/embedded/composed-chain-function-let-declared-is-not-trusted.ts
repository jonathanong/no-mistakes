import { query } from "@data-stores/psql";

function build() {
  return "SELECT id FROM topics";
}

let sql = build();

export function load() {
  return query(sql);
}
