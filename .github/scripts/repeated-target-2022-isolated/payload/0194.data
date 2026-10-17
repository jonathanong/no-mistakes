import { query } from "@example/db";

function build() {
  return "SELECT id FROM topics";
}

const holder = {};
holder.build = "unrelated";

const sql = build();

export function load() {
  return query(sql);
}
