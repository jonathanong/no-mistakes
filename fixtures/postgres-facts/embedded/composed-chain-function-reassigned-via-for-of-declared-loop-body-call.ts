import { query } from "@data-stores/psql";

function build() {
  return "SELECT id FROM topics";
}

function externalBuilder() {
  return "UNTRUSTED";
}

export function load() {
  for (const build of [externalBuilder]) {
    return query(build());
  }
}
