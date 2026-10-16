import { query } from "@example/db";

function build() {
  return "SELECT id FROM topics";
}

function externalBuilder() {
  return "UNTRUSTED";
}

[build = externalBuilder] = [];

const sql = build();

export function load() {
  return query(sql);
}
