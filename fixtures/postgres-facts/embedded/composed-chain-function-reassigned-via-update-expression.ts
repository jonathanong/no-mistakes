import { query } from "@data-stores/psql";

function build() {
  return "SELECT id FROM topics";
}

build++;

const sql = build();

export function load() {
  return query(sql);
}
