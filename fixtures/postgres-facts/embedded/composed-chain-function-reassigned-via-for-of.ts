import { query } from "@data-stores/psql";

function build() {
  return "SELECT id FROM topics";
}

function externalBuilder() {
  return "UNTRUSTED";
}

for (build of [externalBuilder]) {
  break;
}

const sql = build();

export function load() {
  return query(sql);
}
