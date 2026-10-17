import { query } from "@example/db";

function build() {
  return "SELECT id FROM topics";
}

function externalBuilder() {
  return "UNTRUSTED";
}

const providers = [externalBuilder];
[build] = providers;

const sql = build();

export function load() {
  return query(sql);
}
