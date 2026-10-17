import { query } from "@example/db";

function build() {
  return "SELECT id FROM topics";
}

let sql = build();

export function load() {
  return query(sql);
}
